use bevy_egui::egui::LayerId;

use super::*;

pub struct RepresentationPlugin;

impl Plugin for RepresentationPlugin {
    fn build(&self, _app: &mut App) {}
}

impl RepresentationPlugin {
    fn paint_rect(rect: Rect, ctx: &ClientContext, m: &Material, ui: &mut Ui) -> NodeResult<()> {
        let mut p = Painter::new(rect, ui.ctx());
        if let Ok(color) = ctx.get_var(VarName::color).get_color() {
            p.color = color;
        }
        for a in &m.0 {
            a.paint(ctx, &mut p, ui).track()?
        }
        PainterAction::paint.paint(ctx, &mut p, ui).track()
    }
}

pub trait MaterialPaint {
    fn paint(&self, rect: Rect, ctx: &ClientContext, ui: &mut Ui);
    fn paint_viewer(&self, ctx: &ClientContext, ui: &mut Ui) -> Response;
}

impl MaterialPaint for Material {
    fn paint(&self, rect: Rect, ctx: &ClientContext, ui: &mut Ui) {
        match RepresentationPlugin::paint_rect(rect, ctx, self, ui) {
            Ok(_) => {}
            Err(e) => {
                ui.scope_builder(
                    UiBuilder::new().layer_id(LayerId::debug()).max_rect(rect),
                    |ui| {
                        "[red [b (e)]]".cstr().button(ui).on_hover_ui(|ui| {
                            ui.vertical(|ui| {
                                e.ui(ui);
                                for l in ctx.layers() {
                                    format!("{l:?}").label(ui);
                                }
                            });
                        });
                    },
                );
            }
        };
    }
    fn paint_viewer(&self, ctx: &ClientContext, ui: &mut Ui) -> Response {
        let size_id = ui.id().with("view size");
        let mut size = ui.ctx().data_mut(|w| *w.get_temp_mut_or(size_id, 100.0));
        if DragValue::new(&mut size).ui(ui).changed() {
            size = size.at_least(10.0);
            ui.ctx().data_mut(|w| w.insert_temp(size_id, size));
        }
        let (rect, response) = ui.allocate_exact_size(egui::vec2(size, size), Sense::hover());
        self.paint(rect, ctx, ui);
        ui.painter().rect_stroke(
            rect,
            0,
            Stroke::new(1.0, subtle_borders_and_separators()),
            egui::StrokeKind::Middle,
        );
        response
    }
}
