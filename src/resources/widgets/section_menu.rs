use super::*;

pub struct SectionMenu {
    sections: Vec<GameSection>,
}

#[derive(Default)]
struct GameSection {
    name: &'static str,
    target_state: GameState,
    inner_states: Vec<GameState>,
    options: Vec<GameOption>,
    indicator: Option<fn(&World) -> bool>,
}

impl Default for SectionMenu {
    fn default() -> Self {
        Self {
            sections: [
                GameSection {
                    name: "TITLE",
                    target_state: GameState::Title,
                    ..default()
                },
                GameSection {
                    name: "META",
                    target_state: GameState::Meta,
                    options: [GameOption::Login].into(),
                    ..default()
                },
                GameSection {
                    name: "PLAYERS",
                    target_state: GameState::Players,
                    options: [GameOption::Login].into(),
                    ..default()
                },
                GameSection {
                    name: "GAME",
                    target_state: GameState::GameStart,
                    inner_states: [GameState::Battle, GameState::Shop].into(),
                    options: [GameOption::Login].into(),
                    ..default()
                },
                GameSection {
                    name: "QUESTS",
                    target_state: GameState::Quests,
                    inner_states: [GameState::Quests].into(),
                    options: [GameOption::Login].into(),
                    indicator: Some(|_| {
                        QuestPlugin::have_completed() || QuestPlugin::new_available()
                    }),
                },
                GameSection {
                    name: "INBOX",
                    target_state: GameState::Inbox,
                    inner_states: default(),
                    options: [GameOption::Login].into(),
                    ..default()
                },
                GameSection {
                    name: "STATS",
                    target_state: GameState::Stats,
                    inner_states: default(),
                    options: [GameOption::Login].into(),
                    ..default()
                },
                GameSection {
                    name: "INCUBATOR",
                    target_state: GameState::Incubator,
                    inner_states: default(),
                    options: [GameOption::Login].into(),
                    ..default()
                },
                GameSection {
                    name: "EDITOR",
                    target_state: GameState::Editor,
                    inner_states: default(),
                    options: [GameOption::Login].into(),
                    ..default()
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
                bottom: 3.0,
                ..default()
            }))
            .resizable(false)
            .show_separator_line(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let target = GameState::get_target();
                    let current = cur_state(world);
                    ui.visuals_mut().widgets.hovered.fg_stroke.color = VISIBLE_BRIGHT;
                    let ph = gt().play_head();
                    const TICK: f32 = 3.0;
                    let blink = (ph / TICK).fract() * 2.0;
                    let ticked = gt().ticked(TICK);
                    for GameSection {
                        name,
                        target_state,
                        inner_states,
                        options,
                        indicator,
                    } in self.sections
                    {
                        let active = target.eq(&target_state) || inner_states.contains(&current);
                        let enabled = active || options.iter().all(|o| o.is_fulfilled(world));
                        let mut show_indicator = false;
                        if enabled {
                            if let Some(indicator) = indicator {
                                if ticked {
                                    set_context_bool(world, name, indicator(world));
                                }
                                show_indicator = get_context_bool(world, name);
                            }
                        }
                        let color = if active {
                            YELLOW
                        } else if enabled {
                            VISIBLE_LIGHT
                        } else {
                            VISIBLE_DARK
                        };
                        let resp = Button::click(name.to_owned())
                            .enabled(enabled)
                            .color(color, ui)
                            .min_width(100.0)
                            .ui(ui);
                        if resp.clicked() {
                            target_state.proceed_to_target(world);
                        }
                        if show_indicator {
                            let center = resp.rect.right_center() - egui::vec2(10.0, 0.0);
                            let radius = 4.0;
                            ui.painter().circle(center, radius, YELLOW, Stroke::NONE);
                            if blink < 1.0 {
                                ui.painter().circle_stroke(
                                    center,
                                    blink * 10.0,
                                    Stroke::new((1.0 - blink) * 3.0, YELLOW),
                                );
                            }
                        }
                        ui.painter().line_segment(
                            [
                                resp.rect.right_top() + egui::vec2(5.0, -2.0),
                                resp.rect.right_bottom() + egui::vec2(5.0, 2.0),
                            ],
                            Stroke { width: 1.0, color },
                        );
                    }
                    if let Some(wallet) = TWallet::get_current() {
                        ui.add_space(20.0);
                        wallet
                            .amount
                            .to_string()
                            .cstr_c(VISIBLE_LIGHT)
                            .push(format!(" {CREDITS_SYM}").cstr_c(YELLOW))
                            .style(CstrStyle::Bold)
                            .label(ui);
                    }
                });
            });
    }
}
