use super::*;

pub struct StateMenu {
    buttons: Vec<(&'static str, GameState)>,
}

impl Default for StateMenu {
    fn default() -> Self {
        Self {
            buttons: vec![
                ("TITLE", GameState::Title),
                ("GAME", GameState::CustomBattle),
                ("TEST", GameState::TestScenariosRun),
            ],
        }
    }
}

impl StateMenu {
    pub fn ui(self, ui: &mut Ui, world: &mut World) {
        TopBottomPanel::top("State Menu")
            .frame(Frame::none().inner_margin(Margin::same(13.0)))
            .resizable(false)
            .show_separator_line(false)
            .show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    let target = GameState::get_target();
                    ui.visuals_mut().widgets.hovered.fg_stroke.color = WHITE;
                    for (name, state) in self.buttons {
                        let enabled = state.eq(&target);
                        ui.visuals_mut().widgets.inactive.fg_stroke.color =
                            if enabled { YELLOW } else { GRAY };
                        let resp = egui::Button::new(name)
                            .min_size(egui::vec2(100.0, 0.0))
                            .ui(ui);
                        if resp.clicked() {
                            state.change(world);
                        }
                        ui.painter().line_segment(
                            [
                                resp.rect.right_top() + egui::vec2(5.0, -2.0),
                                resp.rect.right_bottom() + egui::vec2(5.0, 2.0),
                            ],
                            ui.visuals().widgets.inactive.fg_stroke,
                        );
                    }
                });
            });
    }
}
