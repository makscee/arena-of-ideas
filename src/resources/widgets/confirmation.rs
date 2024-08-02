use egui::Sense;

use super::*;

#[derive(Clone)]
pub struct Confirmation {
    text: Cstr,
    accept: fn(&mut World),
    decline: fn(&mut World),
}

fn current_id() -> Id {
    static TILES_DATA: OnceCell<Id> = OnceCell::new();
    *TILES_DATA.get_or_init(|| Id::new("tiles_data"))
}

impl Confirmation {
    #[must_use]
    pub fn new(text: Cstr, accept: fn(&mut World)) -> Self {
        Self {
            text,
            accept,
            decline: |_| {},
        }
    }
    pub fn ui(self, ui: &mut Ui, world: &mut World) {
        center_window("Confirmation window", ui, |ui| {
            ui.vertical_centered_justified(|ui| {
                self.text.as_label(ui).wrap(true).ui(ui);
            });
            ui.columns(2, |ui| {
                ui[0].vertical_centered_justified(|ui| {
                    if Button::click("Decline".into()).red(ui).ui(ui).clicked() {
                        (self.decline)(world);
                        Self::clear(ui.ctx());
                    }
                });
                ui[1].vertical_centered_justified(|ui| {
                    if Button::click("Accept".into()).ui(ui).clicked() {
                        (self.accept)(world);
                        Self::clear(ui.ctx());
                    }
                });
            })
        })
    }

    pub fn add(self, ctx: &egui::Context) {
        ctx.data_mut(|w| w.insert_temp(current_id(), self));
    }
    pub fn clear(ctx: &egui::Context) {
        ctx.data_mut(|w| w.remove_by_type::<Confirmation>());
    }
    pub fn show_current(ctx: &egui::Context, world: &mut World) {
        let Some(c) = ctx.data(|r| r.get_temp::<Confirmation>(current_id())) else {
            return;
        };

        let rect = ctx.screen_rect();
        CentralPanel::default()
            .frame(Frame::none())
            .show(ctx, |ui| {
                ui.allocate_rect(rect, Sense::click_and_drag());
                ui.painter_at(rect).rect_filled(
                    rect,
                    Rounding::ZERO,
                    Color32::from_black_alpha(180),
                );
                c.ui(ui, world);
            });
    }
}
