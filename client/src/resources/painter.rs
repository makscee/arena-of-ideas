use emath::{Rot2, TSTransform};
use epaint::{
    CircleShape, CubicBezierShape, RectShape, TessellationOptions, Tessellator, TextShape,
};

use super::*;
use schema::PainterAction;

pub struct Painter {
    pub rect: Rect,
    pub color: Color32,
    pub hollow: Option<f32>,
    pub mesh: egui::Mesh,
    pub tesselator: Tessellator,
}

impl Painter {
    pub fn new(rect: Rect, ctx: &egui::Context) -> Self {
        Self {
            rect,
            color: high_contrast_text(),
            mesh: egui::Mesh::default(),
            tesselator: new_tesselator(0.0, ctx),
            hollow: None,
        }
    }
}

fn new_tesselator(feathering: f32, ctx: &egui::Context) -> Tessellator {
    let (feathering, feathering_size_in_pixels) = if feathering > 0.0 {
        (true, feathering)
    } else {
        (false, 0.0)
    };
    Tessellator::new(
        ctx.pixels_per_point(),
        TessellationOptions {
            feathering,
            feathering_size_in_pixels,
            ..default()
        },
        ctx.fonts(|r| r.font_image_size()),
        default(),
    )
}

pub trait Paint {
    fn paint_err(&self, context: &ClientContext, p: &mut Painter, ui: &mut Ui) -> NodeResult<bool>;

    fn paint(&self, context: &ClientContext, p: &mut Painter, ui: &mut Ui) {
        match self.paint_err(context, p, ui) {
            Ok(_) => {}
            Err(e) => {
                log::error!("Paint error: {}", e);
            }
        }
    }

    fn paint_viewer(&self, ctx: &ClientContext, ui: &mut Ui) -> Response {
        ui.label("Paint")
    }
}

impl Paint for Material {
    fn paint_err(&self, ctx: &ClientContext, p: &mut Painter, ui: &mut Ui) -> NodeResult<bool> {
        let mut scope = ::rhai::Scope::new();
        scope.push("painter", Vec::<PainterAction>::new());
        if let Ok(t) = ctx.get_var(VarName::t).get_f32() {
            scope.push("t", t);
        }

        // Execute the Rhai script to get painter actions
        let actions = match self.0.execute(scope, ctx) {
            Ok(actions) => actions,
            Err(err) => {
                log::error!("Painter script execution error: {}", err);
                return Ok(false);
            }
        }
        .0;

        // Process each action
        for action in actions {
            if process_painter_action(action, ctx, p, ui)? {
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn paint_viewer(&self, ctx: &ClientContext, ui: &mut Ui) -> Response {
        let mut rect = ui.available_rect_before_wrap();
        rect.set_height(200.0);
        let response = ui.allocate_rect(rect, egui::Sense::hover());

        let mut painter = Painter::new(rect, ui.ctx());
        if let Ok(color) = ctx.get_var(VarName::color).get_color() {
            painter.color = color;
        }
        let _ = self.paint_err(ctx, &mut painter, ui);

        response
    }
}

fn process_painter_action(
    action: PainterAction,
    ctx: &ClientContext,
    p: &mut Painter,
    ui: &mut Ui,
) -> NodeResult<bool> {
    let r = p.rect;
    let up = r.width().min(r.height()) * 0.5;

    match action {
        PainterAction::Circle { radius } => {
            let radius = radius * up;
            let shape = if let Some(width) = p.hollow {
                CircleShape::stroke(default(), radius, Stroke::new(width, p.color))
            } else {
                CircleShape::filled(default(), radius, p.color)
            };
            p.tesselator.tessellate_circle(shape, &mut p.mesh);
        }
        PainterAction::Rectangle { width, height } => {
            let size = Vec2::new(width, height) * 2.0;
            let rect = Rect::from_center_size(default(), size.to_evec2() * up);
            let shape = if let Some(width) = p.hollow {
                RectShape::new(
                    rect,
                    CornerRadius::ZERO,
                    TRANSPARENT,
                    Stroke::new(width, p.color),
                    egui::StrokeKind::Middle,
                )
            } else {
                RectShape::new(
                    rect,
                    CornerRadius::ZERO,
                    p.color,
                    Stroke::NONE,
                    egui::StrokeKind::Middle,
                )
            };
            p.tesselator.tessellate_rect(&shape, &mut p.mesh)
        }
        PainterAction::Curve {
            thickness,
            curvature,
        } => {
            let start = ctx
                .get_var(VarName::position)
                .track()?
                .get_vec2()?
                .to_evec2()
                * up;
            let end = ctx
                .get_var(VarName::extra_position)
                .track()?
                .get_vec2()?
                .to_pos2()
                * up
                - start;
            let thickness = thickness * up;
            let curvature = curvature * up;
            let stroke = Stroke::new(thickness, p.color);
            let curve = CubicBezierShape::from_points_stroke(
                [
                    egui::Pos2::ZERO,
                    egui::pos2(0.0, -curvature),
                    end - egui::vec2(0.0, curvature),
                    end,
                ],
                false,
                TRANSPARENT,
                stroke,
            );
            p.tesselator.tessellate_cubic_bezier(&curve, &mut p.mesh);
        }
        PainterAction::Text { text } => {
            let text = text.cstr_c(p.color).galley(p.color.a() as f32 / 255.0, ui);
            let mut mesh = egui::Mesh::default();
            p.tesselator.tessellate_text(
                &TextShape::new(text.rect.size().to_pos2() * -0.5, text, MISSING_COLOR),
                &mut mesh,
            );
            mesh.transform(TSTransform::from_scaling(up / 40.0));
            p.mesh.append(mesh);
        }
        PainterAction::Hollow { width } => {
            if width > 0.0 {
                p.hollow = Some(width);
            } else {
                p.hollow = None;
            }
        }
        PainterAction::Solid => {
            p.hollow = None;
        }
        PainterAction::Translate { x, y } => {
            p.mesh.translate(Vec2::new(x, y).to_evec2() * up);
        }
        PainterAction::Rotate { angle } => {
            p.mesh.rotate(Rot2::from_angle(angle), default());
        }
        PainterAction::ScaleMesh { scale } => {
            p.mesh.transform(TSTransform::from_scaling(scale));
        }
        PainterAction::ScaleRect { scale } => {
            let size = p.rect.size() * scale - p.rect.size();
            p.rect = p.rect.expand2(size * 0.5);
        }
        PainterAction::Color { color } => {
            p.color = color;
        }
        PainterAction::Alpha { alpha } => {
            p.color = p.color.gamma_multiply(alpha.clamp(0.0, 1.0));
        }
        PainterAction::Feathering { amount } => {
            if !p.mesh.is_empty() {
                p.mesh.translate(r.center().to_vec2());
                ui.painter().add(mem::take(&mut p.mesh));
            }
            p.tesselator = new_tesselator(amount, ui.ctx());
        }
        PainterAction::Paint => {
            if p.mesh.is_empty() {
                return Ok(false);
            }
            p.mesh.translate(r.center().to_vec2());
            ui.painter().add(mem::take(&mut p.mesh));
        }
        PainterAction::Exit => {
            return Ok(true);
        }
    };
    Ok(false)
}
