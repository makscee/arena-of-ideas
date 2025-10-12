use emath::{Rot2, TSTransform};
use epaint::{
    CircleShape, CubicBezierShape, RectShape, TessellationOptions, Tessellator, TextShape,
};

use super::*;

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
    fn paint(&self, context: &ClientContext, p: &mut Painter, ui: &mut Ui) -> NodeResult<()>;
}

impl Paint for PainterAction {
    fn paint(&self, ctx: &ClientContext, p: &mut Painter, ui: &mut Ui) -> NodeResult<()> {
        let r = p.rect;
        let up = r.width().min(r.height()) * 0.5;
        ctx.with_temp_layers(default(), |ctx| {
            match self {
                PainterAction::circle(x) => {
                    let radius = x.get_f32(ctx)? * up;
                    let shape = if let Some(width) = p.hollow {
                        CircleShape::stroke(default(), radius, Stroke::new(width, p.color))
                    } else {
                        CircleShape::filled(default(), radius, p.color)
                    };
                    p.tesselator.tessellate_circle(shape, &mut p.mesh)
                }
                PainterAction::rectangle(x) => {
                    let size = x.get_vec2(ctx)? * 2.0;
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
                PainterAction::curve {
                    thickness,
                    curvature,
                } => {
                    let start = ctx.get_var(VarName::position)?.get_vec2()?.to_evec2() * up;
                    let end =
                        ctx.get_var(VarName::extra_position)?.get_vec2()?.to_pos2() * up - start;
                    let thickness = thickness.get_f32(ctx)? * up;
                    let curvature = curvature.get_f32(ctx)? * up;
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
                PainterAction::text(x) => {
                    let text = x
                        .get_string(ctx)?
                        .cstr_c(p.color)
                        .galley(p.color.a() as f32 / 255.0, ui);
                    let mut mesh = egui::Mesh::default();
                    p.tesselator.tessellate_text(
                        &TextShape::new(text.rect.size().to_pos2() * -0.5, text, MISSING_COLOR),
                        &mut mesh,
                    );
                    mesh.transform(TSTransform::from_scaling(up / 40.0));
                    p.mesh.append(mesh);
                }
                PainterAction::hollow(x) => {
                    let x = x.get_f32(ctx)?;
                    if x > 0.0 {
                        p.hollow = Some(x);
                    } else {
                        p.hollow = None;
                    }
                }
                PainterAction::translate(x) => {
                    p.mesh.translate(x.get_vec2(ctx)?.to_evec2() * up);
                }
                PainterAction::rotate(x) => {
                    p.mesh.rotate(Rot2::from_angle(x.get_f32(ctx)?), default());
                }
                PainterAction::scale_mesh(x) => {
                    p.mesh.transform(TSTransform::from_scaling(x.get_f32(ctx)?));
                }
                PainterAction::scale_rect(x) => {
                    let size = p.rect.size() * x.get_f32(ctx)? - p.rect.size();
                    p.rect = p.rect.expand2(size * 0.5);
                }
                PainterAction::color(x) => {
                    p.color = x.get_color(ctx)?;
                }
                PainterAction::alpha(x) => {
                    p.color = p.color.gamma_multiply(x.get_f32(ctx)?.clamp(0.0, 1.0));
                }
                PainterAction::feathering(x) => {
                    Self::paint.paint(ctx, p, ui)?;
                    let x = x.get_f32(ctx)?;
                    p.tesselator = new_tesselator(x, ui.ctx());
                }
                PainterAction::repeat(x, action) => {
                    let max_index = x.get_i32(ctx)?;
                    for i in 0..max_index {
                        ctx.with_layers_temp(
                            [
                                ContextLayer::Var(VarName::index, i.into()),
                                ContextLayer::Var(VarName::max_index, max_index.into()),
                            ]
                            .into(),
                            |context| action.paint(context, p, ui),
                        )
                        .ui(ui);
                    }
                }
                PainterAction::paint => {
                    if p.mesh.is_empty() {
                        return Ok(());
                    }
                    p.mesh.translate(r.center().to_vec2());
                    ui.painter().add(mem::take(&mut p.mesh));
                }
                PainterAction::list(l) => {
                    for a in l {
                        a.paint(ctx, p, ui)?;
                    }
                }
            };
            Ok(())
        })
    }
}
