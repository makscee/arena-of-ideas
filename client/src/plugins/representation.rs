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
        // Execute the Material's Rhai script
        m.paint_err(ctx, &mut p, ui)?;
        Ok(())
    }
}

pub trait MaterialPaint {
    fn paint(&self, rect: Rect, ctx: &ClientContext, ui: &mut Ui);
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
                                self.0.code.label(ui);
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
}
