use super::*;

use egui::{
    emath::{Rot2, TSTransform},
    epaint, Vec2,
};
use epaint::{CircleShape, RectShape, Tessellator, TextShape};

pub struct Painter {
    pub rect: Rect,
    pub color: Color32,
    pub mesh: egui::Mesh,
    pub tesselator: Tessellator,
}

pub trait Paint {
    fn paint(&self, context: &Context, p: &mut Painter, ui: &mut Ui)
        -> Result<(), ExpressionError>;
}

impl Paint for PainterAction {
    fn paint(
        &self,
        context: &Context,
        p: &mut Painter,
        ui: &mut Ui,
    ) -> Result<(), ExpressionError> {
        let r = ui.available_rect_before_wrap();
        match self {
            PainterAction::Circle(x) => {
                let radius = x.get_f32(context)?;
                p.tesselator.tessellate_circle(
                    CircleShape::stroke(r.center(), radius, Stroke::new(2.0, p.color)),
                    &mut p.mesh,
                )
            }
            PainterAction::Rectangle(x) => {
                let size = x.get_vec2(context)?;
                p.tesselator.tessellate_rect(
                    &RectShape::new(
                        Rect::from_center_size(r.center(), size.to_evec2()),
                        Rounding::ZERO,
                        p.color,
                        Stroke::NONE,
                    ),
                    &mut p.mesh,
                )
            }
            PainterAction::Text(x) => {
                let text = x.get_string(context)?;
                p.tesselator.tessellate_text(
                    &TextShape::new(r.center(), text.galley(ui), MISSING_COLOR),
                    &mut p.mesh,
                )
            }
            PainterAction::Hollow(x) => todo!(),
            PainterAction::Translate(x) => {
                p.mesh.translate(x.get_vec2(context)?.to_evec2());
            }
            PainterAction::Scale(x) => {
                p.mesh
                    .transform(TSTransform::from_scaling(x.get_f32(context)?));
            }
            PainterAction::Color(x) => {
                p.color = x.get_color(context)?;
            }
            PainterAction::Repeat(x, action) => {
                for _ in 0..x.get_i32(context)? {
                    action.paint(context, p, ui);
                }
            }
        };
        Ok(())
    }
}
