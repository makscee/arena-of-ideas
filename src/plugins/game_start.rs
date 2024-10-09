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
    runs: Vec<TArenaRunArchive>,
    show_leaderboard: bool,
    teams: Vec<TTeam>,
    selected_team: usize,
}

fn rm(world: &mut World) -> Mut<GameStartResource> {
    world.resource_mut::<GameStartResource>()
}
fn r(world: &World) -> &GameStartResource {
    world.resource::<GameStartResource>()
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
            runs: default(),
            show_leaderboard: true,
            teams: TTeam::filter_by_owner(user_id())
                .filter(|t| t.pool == TeamPool::Owned && !t.units.is_empty())
                .collect_vec(),
            selected_team: 0,
        }
    }
}

impl GameStartPlugin {
    pub fn add_tiles(world: &mut World) {
        let gsr = rm(world);
        Self::load_leaderboard(gsr.game_modes[gsr.selected].clone(), world);
        Tile::new(Side::Bottom, |ui, world| {
            ui.vertical_centered(|ui| {
                let gsr = rm(world);
                let game_mode = gsr.game_modes[gsr.selected].clone();
                let can_start = !GameOption::ActiveRun.is_fulfilled(world);
                ui.vertical_centered_justified(|ui| {
                    ui.set_width(400.0);
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
                            let cost = TPrices::current().ranked_mode;
                            let gsr = r(world);
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
                                        let gsr = rm(world);
                                        let team = gsr.teams[gsr.selected_team].clone();
                                        team.hover_label(ui, world);
                                    },
                                    |ui, world| {
                                        if Button::click("<".to_owned())
                                            .min_width(ARROW_WIDTH)
                                            .ui(ui)
                                            .clicked()
                                        {
                                            let mut gsr = rm(world);
                                            gsr.selected_team =
                                                (gsr.selected_team + gsr.teams.len() - 1)
                                                    % gsr.teams.len();
                                        }
                                    },
                                    |ui, world| {
                                        if Button::click(">".to_owned())
                                            .min_width(ARROW_WIDTH)
                                            .ui(ui)
                                            .clicked()
                                        {
                                            let mut gsr = rm(world);
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
                                    let gsr = r(world);
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
                            let cost = TPrices::current().const_mode;
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
                            seed.cstr_cs(VISIBLE_LIGHT, CstrStyle::Bold).label(ui);
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
                ui.add_space(30.0);
                "Game Mode".cstr().label(ui);
                const ARROW_WIDTH: f32 = 100.0;
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
                            if Button::click("<".to_owned())
                                .min_width(ARROW_WIDTH)
                                .ui(ui)
                                .clicked()
                            {
                                let mut gsr = rm(world);
                                gsr.selected = (gsr.selected + gsr.game_modes.len() - 1)
                                    % gsr.game_modes.len();
                                Self::load_leaderboard(gsr.game_modes[gsr.selected].clone(), world);
                            }
                        },
                        |ui, world| {
                            if Button::click(">".to_owned())
                                .min_width(ARROW_WIDTH)
                                .ui(ui)
                                .clicked()
                            {
                                let mut gsr = rm(world);
                                gsr.selected = (gsr.selected + 1) % gsr.game_modes.len();
                                Self::load_leaderboard(gsr.game_modes[gsr.selected].clone(), world);
                            }
                        },
                    );
            });
        })
        .pinned()
        .push(world);
        Tile::new(Side::Left, |ui, world| {
            world.resource_scope(|world, mut gsr: Mut<GameStartResource>| {
                ui.horizontal(|ui| {
                    if Button::click("Leaderboard".into())
                        .active(gsr.show_leaderboard)
                        .ui(ui)
                        .clicked()
                    {
                        gsr.show_leaderboard = true;
                    }
                    if Button::click("Runs".into())
                        .active(!gsr.show_leaderboard)
                        .ui(ui)
                        .clicked()
                    {
                        gsr.show_leaderboard = false;
                    }
                });
                if gsr.show_leaderboard {
                    gsr.leaderboard.show_table("Leaderboard", ui, world);
                } else {
                    gsr.runs.show_table("Runs", ui, world);
                }
            });
        })
        .pinned()
        .push(world);
        if GameOption::ActiveRun.is_fulfilled(world) {
            Tile::new(Side::Right, move |ui, world| {
                if let Some(run) = TArenaRun::get_current() {
                    ui.vertical_centered(|ui| {
                        ShopPlugin::show_stats(&run, ui);
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
            .pinned()
            .no_expand()
            .push(world);
        }
        Tile::new(Side::Left, |ui, world| {
            let gsr = rm(world);
            let game_mode = gsr.game_modes[gsr.selected].clone();
            match game_mode {
                GameMode::ArenaNormal => {
                    "1. Defeat as many enemies as possible\n\
                    2. 3 lives, replenish every 5 floors\n\
                    3. Defeat current champion for big reward\n\
                    4. Credits reward depending on win streak"
                        .cstr_c(VISIBLE_LIGHT)
                        .label(ui);
                }
                GameMode::ArenaRanked => {
                    "1. Start with own team\n\
                    2. Defeat as many enemies as possible\n\
                    3. 3 lives, replenish every 5 floors\n\
                    4. Defeat current champion for big reward\n\
                    5. Credits reward depending on win streak\n\
                    6. Rewards are multiplied by 2"
                        .cstr_c(VISIBLE_LIGHT)
                        .label(ui);
                }
                GameMode::ArenaConst(_) => {
                    "1. Defeat as many enemies as possible\n\
                    2. Entry fee growing every time, reset on day start\n\
                    3. Credits reward depending on win streak\n\
                    4. Fixed seed, new seed after reaching 10 floors\n\
                    5. 3 lives, replenish every 5 floors\n\
                    6. Defeat current champion for big reward\n\
                    7. Rewards are multiplied by 3"
                        .cstr_c(VISIBLE_LIGHT)
                        .inject_color(YELLOW)
                        .label(ui);
                }
            }
        })
        .transparent()
        .pinned()
        .no_expand()
        .push(world);
    }

    fn load_leaderboard(game_mode: GameMode, world: &mut World) {
        TableState::reset_cache(&egui_context(world).unwrap());
        let mut gsr = rm(world);
        gsr.leaderboard = TArenaLeaderboard::iter()
            .filter(|d| d.mode.eq(&game_mode))
            .sorted_by_key(|d| -(d.floor as i32))
            .collect_vec();
        gsr.runs = TArenaRunArchive::iter()
            .filter(|d| d.mode.eq(&game_mode))
            .sorted_by_key(|d| -(d.id as i32))
            .collect_vec();
    }
}
