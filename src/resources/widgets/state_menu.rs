use super::*;

pub struct StateMenu {
    buttons: Vec<(&'static str, GameState)>,
}

impl Default for StateMenu {
    fn default() -> Self {
        Self {
            buttons: vec![
                ("TITLE", GameState::Title),
                (user_name(), GameState::Profile),
                ("SHOP", GameState::Shop),
                ("GAME", GameState::CustomBattle),
                ("LEADERBOARD", GameState::TableView(QUERY_LEADERBOARD)),
                ("TEST", GameState::TestScenariosRun),
            ],
        }
    }
}

impl StateMenu {
    pub fn show(self, ctx: &egui::Context, world: &mut World) {
        TopBottomPanel::top("State Menu")
            .frame(Frame::none().outer_margin(Margin {
                left: 13.0,
                top: 3.0,
                ..default()
            }))
            .resizable(false)
            .show_separator_line(false)
            .show(ctx, |ui| {
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
                            state.proceed_to_target(world);
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
