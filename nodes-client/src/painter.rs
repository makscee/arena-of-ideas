use std::mem;

use super::*;

use bevy_egui::egui::{self, Rect, Rounding, Stroke, Ui};
use egui::{
    emath::{Rot2, TSTransform},
    epaint::{self, TessellationOptions},
    Mesh,
};
use epaint::{CircleShape, RectShape, Tessellator, TextShape};

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
            color: VISIBLE_LIGHT,
            mesh: Mesh::default(),
            tesselator: Tessellator::new(
                ctx.pixels_per_point(),
                TessellationOptions::default(),
                ctx.fonts(|r| r.font_image_size()),
                default(),
            ),
            hollow: None,
        }
    }
}

pub trait Paint {
    fn paint(
        &self,
        context: &mut Context,
        p: &mut Painter,
        ui: &mut Ui,
    ) -> Result<(), ExpressionError>;
}

impl Paint for PainterAction {
    fn paint(
        &self,
        context: &mut Context,
        p: &mut Painter,
        ui: &mut Ui,
    ) -> Result<(), ExpressionError> {
        let r = ui.available_rect_before_wrap();
        let up = unit_pixels();
        match self {
            PainterAction::Circle(x) => {
                let radius = x.get_f32(context)? * up;
                let shape = if let Some(width) = p.hollow {
                    CircleShape::stroke(default(), radius, Stroke::new(width, p.color))
                } else {
                    CircleShape::filled(default(), radius, p.color)
                };
                p.tesselator.tessellate_circle(shape, &mut p.mesh)
            }
            PainterAction::Rectangle(x) => {
                let size = x.get_vec2(context)? * 2.0;
                let rect = Rect::from_center_size(default(), size.to_evec2() * up);
                let shape = if let Some(width) = p.hollow {
                    RectShape::new(
                        rect,
                        Rounding::ZERO,
                        TRANSPARENT,
                        Stroke::new(width, p.color),
                    )
                } else {
                    RectShape::new(rect, Rounding::ZERO, p.color, Stroke::NONE)
                };
                p.tesselator.tessellate_rect(&shape, &mut p.mesh)
            }
            PainterAction::Text(x) => {
                let text = x.get_string(context)?.cstr_c(p.color).galley(ui);
                let mut mesh = Mesh::default();
                p.tesselator.tessellate_text(
                    &TextShape::new(text.rect.size().to_pos2() * -0.5, text, MISSING_COLOR),
                    &mut mesh,
                );
                mesh.transform(TSTransform::from_scaling(up / 40.0));
                p.mesh.append(mesh);
            }
            PainterAction::Hollow(x) => p.hollow = Some(x.get_f32(context)?),
            PainterAction::Translate(x) => {
                p.mesh.translate(x.get_vec2(context)?.to_evec2() * up);
            }
            PainterAction::Rotate(x) => {
                p.mesh
                    .rotate(Rot2::from_angle(x.get_f32(context)?), default());
            }
            PainterAction::Scale(x) => {
                p.mesh
                    .transform(TSTransform::from_scaling(x.get_f32(context)?));
            }
            PainterAction::Color(x) => {
                p.color = x.get_color(context)?;
            }
            PainterAction::Alpha(x) => {
                p.color = p.color.gamma_multiply(x.get_f32(context)?.clamp(0.0, 1.0));
            }
            PainterAction::Repeat(x, action) => {
                for i in 0..x.get_i32(context)? {
                    context.set_var(VarName::index, i.into());
                    action.paint(context, p, ui)?;
                }
            }
            PainterAction::Paint => {
                if p.mesh.is_empty() {
                    return Ok(());
                }
                p.mesh.translate(r.center().to_vec2());
                ui.painter().add(mem::take(&mut p.mesh));
            }
            PainterAction::List(l) => {
                for a in l {
                    a.paint(context, p, ui)?;
                }
            }
        };
        Ok(())
    }
}
