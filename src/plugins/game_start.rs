use super::*;

pub struct GameStartPlugin;

impl Plugin for GameStartPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::GameStart), |world: &mut World| {
            world.init_resource::<GameStartResource>();
        });
    }
}

#[derive(Resource)]
struct GameStartResource {
    game_modes: Vec<GameMode>,
    selected: usize,
}

impl Default for GameStartResource {
    fn default() -> Self {
        Self {
            game_modes: [
                GameMode::ArenaNormal,
                GameMode::ArenaRanked,
                GameMode::ArenaConst(GlobalData::current().constant_seed.clone()),
            ]
            .into(),
            selected: 0,
        }
    }
}

impl GameStartPlugin {
    pub fn add_tiles(world: &mut World) {
        Tile::new(Side::Bottom, |ui, world| {
            if GameOption::ActiveRun.is_fulfilled(world) {
                ui.vertical_centered(|ui| {
                    if Button::click("Continue".into()).ui(ui).clicked() {
                        GameState::Shop.proceed_to_target(world);
                    }
                    if Button::click("Abandon run".into()).red(ui).ui(ui).clicked() {
                        Confirmation::new("Abandon current run?".cstr_c(VISIBLE_BRIGHT), |_| {
                            run_finish();
                        })
                        .add(ui.ctx());
                    }
                });
            } else {
                ui.vertical_centered(|ui| {
                    "Game Mode".cstr().label(ui);
                    const ARROW_WIDTH: f32 = 100.0;
                    let gsr = world.resource_mut::<GameStartResource>();
                    let game_mode = gsr.game_modes[gsr.selected].clone();
                    Middle3::default()
                        .width(400.0)
                        .side_align(Align::Min)
                        .ui_mut(
                            ui,
                            world,
                            |ui, _| {
                                match &game_mode {
                                    GameMode::ArenaNormal => {
                                        "Arena Normal"
                                            .cstr_c(VISIBLE_BRIGHT)
                                            .style(CstrStyle::Heading2)
                                            .label(ui);
                                        if Button::click("Play".into()).ui(ui).clicked() {
                                            run_start_normal();
                                            once_on_run_start_normal(|_, _, status| {
                                                status.on_success(|w| {
                                                    GameState::Shop.proceed_to_target(w)
                                                })
                                            });
                                        }
                                    }
                                    GameMode::ArenaRanked => {
                                        "Arena Ranked"
                                            .cstr_c(YELLOW)
                                            .style(CstrStyle::Heading2)
                                            .label(ui);
                                        if Button::click(format!(
                                            "-{}¤",
                                            GlobalSettings::current().arena.ranked_cost
                                        ))
                                        .title("Play".cstr())
                                        .ui(ui)
                                        .clicked()
                                        {
                                            run_start_ranked();
                                            once_on_run_start_ranked(|_, _, status| {
                                                status.on_success(|w| {
                                                    GameState::Shop.proceed_to_target(w)
                                                })
                                            });
                                        }
                                        "Wallet: "
                                            .cstr()
                                            .push(
                                                format!("{}¤", TWallet::current().amount)
                                                    .cstr_cs(YELLOW, CstrStyle::Bold),
                                            )
                                            .label(ui);
                                    }
                                    GameMode::ArenaConst(seed) => {
                                        "Arena Constant"
                                            .cstr_c(CYAN)
                                            .style(CstrStyle::Heading2)
                                            .label(ui);
                                        seed.cstr_cs(VISIBLE_LIGHT, CstrStyle::Bold).label(ui);
                                    }
                                };
                            },
                            |ui, world| {
                                ui.add_space(50.0);
                                if Button::click("<".to_owned())
                                    .min_width(ARROW_WIDTH)
                                    .ui(ui)
                                    .clicked()
                                {
                                    let mut gsr = world.resource_mut::<GameStartResource>();
                                    gsr.selected = (gsr.selected + gsr.game_modes.len() - 1)
                                        % gsr.game_modes.len();
                                }
                            },
                            |ui, world| {
                                ui.add_space(50.0);
                                if Button::click(">".to_owned())
                                    .min_width(ARROW_WIDTH)
                                    .ui(ui)
                                    .clicked()
                                {
                                    let mut gsr = world.resource_mut::<GameStartResource>();
                                    gsr.selected = (gsr.selected + 1) % gsr.game_modes.len();
                                }
                            },
                        );
                    ui.add_space(30.0);
                });
            }
        })
        .min_size(200.0)
        .transparent()
        .push(world);
    }
}
