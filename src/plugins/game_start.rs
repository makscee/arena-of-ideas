use super::*;

pub struct GameStartPlugin;

impl Plugin for GameStartPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::GameStart), |world: &mut World| {
            world.insert_resource(GameStartResource::default());
        });
    }
}

#[derive(Resource)]
struct GameStartResource {
    game_modes: Vec<GameMode>,
    selected: usize,
    leaderboard: Vec<TArenaLeaderboard>,
    teams: Vec<TTeam>,
    selected_team: usize,
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
            leaderboard: default(),
            teams: TTeam::filter_by_owner(user_id())
                .filter(|t| t.pool == TeamPool::Owned && !t.units.is_empty())
                .collect_vec(),
            selected_team: 0,
        }
    }
}

impl GameStartPlugin {
    pub fn add_tiles(world: &mut World) {
        let gsr = world.resource_mut::<GameStartResource>();
        Self::load_leaderboard(gsr.game_modes[gsr.selected].clone(), world);
        Tile::new(Side::Bottom, |ui, world| {
            ui.vertical_centered(|ui| {
                "Game Mode".cstr().label(ui);
                const ARROW_WIDTH: f32 = 100.0;
                let gsr = world.resource_mut::<GameStartResource>();
                let game_mode = gsr.game_modes[gsr.selected].clone();
                let can_start = !GameOption::ActiveRun.is_fulfilled(world);
                Middle3::default()
                    .width(300.0)
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
                                }
                                GameMode::ArenaRanked => {
                                    "Arena Ranked"
                                        .cstr_c(YELLOW)
                                        .style(CstrStyle::Heading2)
                                        .label(ui);
                                }
                                GameMode::ArenaConst(..) => {
                                    "Arena Constant"
                                        .cstr_c(CYAN)
                                        .style(CstrStyle::Heading2)
                                        .label(ui);
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
                                Self::load_leaderboard(gsr.game_modes[gsr.selected].clone(), world);
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
                                Self::load_leaderboard(gsr.game_modes[gsr.selected].clone(), world);
                            }
                        },
                    );
                match &game_mode {
                    GameMode::ArenaNormal => {
                        if Button::click("Play".into())
                            .enabled(can_start)
                            .ui(ui)
                            .clicked()
                        {
                            run_start_normal();
                            once_on_run_start_normal(|_, _, status| {
                                status.on_success(|w| GameState::Shop.proceed_to_target(w))
                            });
                        }
                    }
                    GameMode::ArenaRanked => {
                        let cost = GlobalSettings::current().arena.ranked_cost_min;
                        let gsr = world.resource::<GameStartResource>();
                        if gsr.teams.is_empty() {
                            "Need at least one non-empty team to play this mode"
                                .cstr_cs(RED, CstrStyle::Bold)
                                .label(ui);
                        } else {
                            Middle3::default().width(300.0).ui_mut(
                                ui,
                                world,
                                |ui, world| {
                                    "Select team".cstr_c(VISIBLE_LIGHT).label(ui);
                                    let gsr = world.resource_mut::<GameStartResource>();
                                    let team = gsr.teams[gsr.selected_team].clone();
                                    if team.cstr().label(ui).hovered() {
                                        cursor_window(ui.ctx(), |ui| {
                                            Frame {
                                                inner_margin: Margin::same(8.0),
                                                rounding: Rounding::same(13.0),
                                                fill: BG_TRANSPARENT,
                                                ..default()
                                            }
                                            .show(
                                                ui,
                                                |ui| {
                                                    team.show(ui, world);
                                                },
                                            );
                                        });
                                    }
                                },
                                |ui, world| {
                                    if Button::click("<".to_owned())
                                        .min_width(ARROW_WIDTH)
                                        .ui(ui)
                                        .clicked()
                                    {
                                        let mut gsr = world.resource_mut::<GameStartResource>();
                                        gsr.selected_team = (gsr.selected_team + gsr.teams.len()
                                            - 1)
                                            % gsr.teams.len();
                                    }
                                },
                                |ui, world| {
                                    if Button::click(">".to_owned())
                                        .min_width(ARROW_WIDTH)
                                        .ui(ui)
                                        .clicked()
                                    {
                                        let mut gsr = world.resource_mut::<GameStartResource>();
                                        gsr.selected_team =
                                            (gsr.selected_team + 1) % gsr.teams.len();
                                    }
                                },
                            );
                            if Button::click(format!("-{} {CREDITS_SYM}", cost))
                                .title("Play".cstr())
                                .enabled(can_start && TWallet::current().amount >= cost)
                                .ui(ui)
                                .clicked()
                            {
                                let gsr = world.resource::<GameStartResource>();
                                let team = gsr.teams[gsr.selected_team].id;
                                run_start_ranked(team);
                                once_on_run_start_ranked(|_, _, status, _| {
                                    status.on_success(|w| GameState::Shop.proceed_to_target(w))
                                });
                            }
                            "Wallet: "
                                .cstr()
                                .push(
                                    format!("{} {CREDITS_SYM}", TWallet::current().amount)
                                        .cstr_cs(YELLOW, CstrStyle::Bold),
                                )
                                .label(ui);
                        }
                    }
                    GameMode::ArenaConst(seed) => {
                        let cost = GlobalSettings::current().arena.ranked_cost_min;
                        seed.cstr_cs(VISIBLE_LIGHT, CstrStyle::Bold).label(ui);
                        if Button::click(format!("-{} {CREDITS_SYM}", cost))
                            .title("Play".cstr())
                            .enabled(can_start && TWallet::current().amount >= cost)
                            .ui(ui)
                            .clicked()
                        {
                            run_start_const();
                            once_on_run_start_const(|_, _, status| {
                                status.on_success(|w| GameState::Shop.proceed_to_target(w))
                            });
                        }
                        "Wallet: "
                            .cstr()
                            .push(
                                format!("{} {CREDITS_SYM}", TWallet::current().amount)
                                    .cstr_cs(YELLOW, CstrStyle::Bold),
                            )
                            .label(ui);
                    }
                }
            });
        })
        .pinned()
        .min_space(egui::vec2(0.0, 220.0))
        .push(world);
        Tile::new(Side::Left, |ui, world| {
            world.resource_scope(|world, gsr: Mut<GameStartResource>| {
                gsr.leaderboard.show_table("Leaderboard", ui, world);
            });
        })
        .pinned()
        .push(world);
        if GameOption::ActiveRun.is_fulfilled(world) {
            Tile::new(Side::Right, move |ui, world| {
                if let Some(run) = TArenaRun::get_current() {
                    ui.vertical_centered(|ui| {
                        text_dots_text("run".cstr(), run.mode.cstr(), ui);
                        text_dots_text(
                            "round".cstr(),
                            run.round.to_string().cstr_c(VISIBLE_LIGHT),
                            ui,
                        );
                        text_dots_text("lives".cstr(), run.lives.to_string().cstr_c(GREEN), ui);
                        text_dots_text(
                            "score".cstr(),
                            run.score.to_string().cstr_c(VISIBLE_BRIGHT),
                            ui,
                        );
                        ui.add_space(20.0);
                        if Button::click("Continue".into()).ui(ui).clicked() {
                            GameState::Shop.proceed_to_target(world);
                        }
                        if Button::click("Abandon run".into()).red(ui).ui(ui).clicked() {
                            Confirmation::new(
                                "Abandon current run?".cstr_c(VISIBLE_BRIGHT),
                                |_| {
                                    run_finish();
                                },
                            )
                            .push(ui.ctx());
                        }
                    });
                }
            })
            .transparent()
            .non_focusable()
            .pinned()
            .push(world);
        }
        Tile::new(Side::Bottom, |ui, world| {
            let gsr = world.resource_mut::<GameStartResource>();
            let game_mode = gsr.game_modes[gsr.selected].clone();
            match game_mode {
                GameMode::ArenaNormal => {
                    "1. Defeat as many enemies as possible\n\
                    2. 3 lives, replenish every 5 floors\n\
                    3. Defeat current champion for big reward\n\
                    4. Credits reward on reaching floor 10"
                        .cstr_c(VISIBLE_LIGHT)
                        .label(ui);
                }
                GameMode::ArenaRanked => {
                    "1. Defeat as many enemies as possible\n\
                    2. Entry fee growing every time, reset on day start\n\
                    3. Get <¤> for each floor beaten\n\
                    4. 3 lives, replenish every 5 floors\n\
                    5. Defeat current champion for big reward"
                        .cstr_c(VISIBLE_LIGHT)
                        .inject_color(YELLOW)
                        .label(ui);
                }
                GameMode::ArenaConst(_) => {
                    "1. Defeat as many enemies as possible\n\
                    2. Entry fee growing every time, reset on day start\n\
                    3. Get <¤> for each floor beaten\n\
                    4. Fixed seed, new seed after reaching 10 floors\n\
                    5. 3 lives, replenish every 5 floors\n\
                    6. Defeat current champion for big reward"
                        .cstr_c(VISIBLE_LIGHT)
                        .inject_color(YELLOW)
                        .label(ui);
                }
            }
        })
        .transparent()
        .pinned()
        .non_focusable()
        .push(world);
    }

    fn load_leaderboard(game_mode: GameMode, world: &mut World) {
        TableState::reset_cache(&egui_context(world).unwrap());
        world.resource_mut::<GameStartResource>().leaderboard = TArenaLeaderboard::iter()
            .filter(|d| d.mode.eq(&game_mode))
            .sorted_by_key(|d| -(d.round as i32))
            .collect_vec();
    }
}
