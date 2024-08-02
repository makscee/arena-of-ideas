use super::*;

pub struct SectionMenu {
    sections: Vec<GameSection>,
}

struct GameSection {
    name: String,
    target_state: GameState,
    inner_states: Vec<GameState>,
    options: Vec<GameOption>,
}

impl Default for SectionMenu {
    fn default() -> Self {
        Self {
            sections: [
                GameSection {
                    name: "TITLE".to_owned(),
                    target_state: GameState::Title,
                    inner_states: default(),
                    options: default(),
                },
                GameSection {
                    name: "GAME".to_owned(),
                    target_state: GameState::Shop,
                    inner_states: [GameState::Battle].into(),
                    options: [GameOption::ActiveRun].into(),
                },
                GameSection {
                    name: "HISTORY".to_owned(),
                    target_state: GameState::TableView(StdbQuery::BattleHistory),
                    inner_states: default(),
                    options: default(),
                },
            ]
            .into(),
        }
    }
}

impl SectionMenu {
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
                    let current = cur_state(world);
                    ui.visuals_mut().widgets.hovered.fg_stroke.color = VISIBLE_BRIGHT;
                    for GameSection {
                        name,
                        target_state,
                        inner_states,
                        options,
                    } in self.sections
                    {
                        let active = target.eq(&target_state) || inner_states.contains(&current);
                        let enabled = active || options.iter().all(|o| o.is_fulfilled(world));
                        let color = if active {
                            YELLOW
                        } else if enabled {
                            VISIBLE_LIGHT
                        } else {
                            VISIBLE_DARK
                        };
                        let resp = Button::click(name)
                            .enabled(enabled)
                            .color(color, ui)
                            .min_width(100.0)
                            .ui(ui);
                        if resp.clicked() {
                            target_state.proceed_to_target(world);
                        }
                        ui.painter().line_segment(
                            [
                                resp.rect.right_top() + egui::vec2(5.0, -2.0),
                                resp.rect.right_bottom() + egui::vec2(5.0, 2.0),
                            ],
                            Stroke { width: 1.0, color },
                        );
                    }
                });
            });
    }
}
