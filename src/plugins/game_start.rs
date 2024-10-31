use super::*;

pub struct GameStartPlugin;

#[derive(Resource)]
struct GameStartResource {
    game_modes: Vec<GameMode>,
    selected_mode: usize,
    leaderboard: HashMap<usize, Vec<TArenaLeaderboard>>,
    selected_season: u32,
    runs: HashMap<usize, Vec<TArenaRunArchive>>,
    teams: Vec<TTeam>,
    selected_team: usize,
}

fn rm(world: &mut World) -> Mut<GameStartResource> {
    world.resource_mut::<GameStartResource>()
}

impl Default for GameStartResource {
    fn default() -> Self {
        let teams = TTeam::filter_by_owner(user_id())
            .filter(|t| t.pool == TeamPool::Owned && !t.units.is_empty())
            .collect_vec();
        let selected_team = client_state()
            .last_played_team
            .and_then(|id| teams.iter().position(|t| t.id == id))
            .unwrap_or_default();
        Self {
            game_modes: [
                GameMode::ArenaNormal,
                GameMode::ArenaRanked,
                GameMode::ArenaConst(default()),
            ]
            .into(),
            selected_mode: client_state().last_played_mode.unwrap_or_default() as usize,
            leaderboard: default(),
            runs: default(),
            teams,
            selected_team,
            selected_season: global_settings().season,
        }
    }
}

impl GameStartPlugin {
    fn load_data(world: &mut World) {
        TableState::reset_cache(&egui_context(world).unwrap());
        let mut gsr = GameStartResource::default();
        gsr.leaderboard = HashMap::from_iter(
            TArenaLeaderboard::filter_by_season(gsr.selected_season)
                .sorted_by_key(|d| -(d.floor as i32))
                .map(|d| (d.mode.clone().into(), d))
                .into_grouping_map()
                .collect(),
        );
        gsr.runs = HashMap::from_iter(
            TArenaRunArchive::filter_by_season(gsr.selected_season)
                .sorted_by_key(|d| -(d.id as i32))
                .map(|d| (d.mode.clone().into(), d))
                .into_grouping_map()
                .collect(),
        );
        world.insert_resource(gsr);
    }
    pub fn add_tiles(world: &mut World) {
        Self::load_data(world);
        Tile::new(Side::Left, |ui, world| {
            world.resource_scope(|world, r: Mut<GameStartResource>| {
                if let Some(data) = r.leaderboard.get(&r.selected_mode) {
                    title("Leaderboard", ui);
                    if let Some(first) = data.get(0) {
                        ui.vertical_centered_justified(|ui| {
                            "Current champion"
                                .cstr_cs(YELLOW, CstrStyle::Bold)
                                .label(ui);
                            first
                                .user
                                .get_user()
                                .name
                                .cstr_cs(VISIBLE_BRIGHT, CstrStyle::Heading2)
                                .label(ui);
                        });
                    }
                    ScrollArea::both().auto_shrink(false).show(ui, |ui| {
                        data.show_table("Leaderboard", ui, world);
                    });
                }
            })
        })
        .stretch_part(0.4)
        .pinned()
        .transparent()
        .push(world);
        Tile::new(Side::Right, |ui, world| {
            world.resource_scope(|world, r: Mut<GameStartResource>| {
                if let Some(data) = r.runs.get(&r.selected_mode) {
                    title("Runs", ui);
                    ScrollArea::both().auto_shrink(false).show(ui, |ui| {
                        data.show_table("Runs", ui, world);
                    });
                }
            })
        })
        .stretch_part(0.4)
        .pinned()
        .transparent()
        .push(world);
        Tile::new(Side::Left, |ui, world| {
            let mut r = rm(world);
            let modes = r.game_modes.clone().into_iter();
            let mut mode: GameMode = r.selected_mode.clone().into();
            ui.add_space(5.0);
            EnumSwitcher::new()
                .style(CstrStyle::Bold)
                .columns()
                .show_iter(&mut mode, modes, ui);
            if EnumSwitcher::new().prefix("Season ".cstr()).show_iter(
                &mut r.selected_season,
                0..=global_settings().season,
                ui,
            ) {
                Self::load_data(world);
                return;
            }
            br(ui);
            r.selected_mode = mode.clone().into();
            ui.vertical_centered_justified(|ui| {
                mode.cstr().style(CstrStyle::Heading).label(ui);
                ui.add_space(30.0);
                let run = TArenaRun::get_current();
                let mut entry_fee = None;
                if run.is_none() {
                    let mut enabled = true;
                    match &mode {
                        GameMode::ArenaNormal => {}
                        GameMode::ArenaRanked => {
                            let mut r = rm(world);
                            if r.teams.is_empty() {
                                "Need to create at least one team"
                                    .cstr_cs(RED, CstrStyle::Bold)
                                    .as_label(ui)
                                    .wrap()
                                    .ui(ui);
                                enabled = false;
                            } else {
                                let cost = TDailyState::current().ranked_cost;
                                let mut new_selected = None;
                                ui.horizontal_wrapped(|ui| {
                                    for (i, team) in r.teams.iter().enumerate() {
                                        if Button::click(i.to_string())
                                            .cstr(team.cstr())
                                            .active(r.selected_team == i)
                                            .ui(ui)
                                            .clicked()
                                        {
                                            new_selected = Some(i);
                                        }
                                    }
                                });
                                if let Some(i) = new_selected {
                                    r.selected_team = i;
                                }
                                ui.add_space(30.0);
                                r.teams[r.selected_team].clone().hover_label(ui, world);

                                if cost == 0 {
                                    "First daily run free!"
                                        .cstr_cs(GREEN, CstrStyle::Bold)
                                        .label(ui);
                                    space(ui);
                                }
                                entry_fee = Some(cost);
                            }
                        }
                        GameMode::ArenaConst(_) => {
                            let cost = TDailyState::current().const_cost;
                            if cost == 0 {
                                "First daily run free!"
                                    .cstr_cs(GREEN, CstrStyle::Bold)
                                    .label(ui);
                                space(ui);
                            }
                            entry_fee = Some(cost);
                        }
                    };
                    let mut btn = Button::click("Play");
                    if let Some(cost) = entry_fee {
                        btn = btn.credits_cost(cost);
                        enabled = enabled && can_afford(cost);
                    }
                    if btn.big().enabled(enabled).ui(ui).clicked() {
                        let r = rm(world);
                        let mut cs = client_state().clone();
                        cs.last_played_mode = Some(r.selected_mode as u64);
                        match &mode {
                            GameMode::ArenaNormal => {
                                run_start_normal();
                                once_on_run_start_normal(|_, _, status| {
                                    status.on_success(|w| GameState::Shop.proceed_to_target(w))
                                });
                            }
                            GameMode::ArenaRanked => {
                                let team = r.teams[r.selected_team].id;
                                cs.last_played_team = Some(team);
                                run_start_ranked(team);
                                once_on_run_start_ranked(|_, _, status, _| {
                                    status.on_success(|w| GameState::Shop.proceed_to_target(w))
                                });
                            }
                            GameMode::ArenaConst(_) => {
                                run_start_const();
                                once_on_run_start_const(|_, _, status| {
                                    status.on_success(|w| GameState::Shop.proceed_to_target(w))
                                });
                            }
                        }
                        cs.save();
                    }
                }
                ui.add_space(13.0);
                if let Some(run) = run {
                    ui.vertical_centered(|ui| {
                        if Button::click("Continue")
                            .cstr("Continue".cstr_cs(VISIBLE_BRIGHT, CstrStyle::Heading))
                            .ui(ui)
                            .clicked()
                        {
                            GameState::Shop.proceed_to_target(world);
                        }
                        if Button::click("Abandon run").red(ui).ui(ui).clicked() {
                            Confirmation::new("Abandon current run?".cstr_c(VISIBLE_BRIGHT))
                                .accept(|_| {
                                    run_finish();
                                })
                                .cancel(|_| {})
                                .push(world);
                        }
                        ui.add_space(20.0);
                        ShopPlugin::show_stats(&run, ui);
                    });
                }
                ui.with_layout(Layout::bottom_up(Align::LEFT), |ui| {
                    match mode {
                        GameMode::ArenaNormal => {
                            "1. Defeat as many enemies as possible\n\
                        2. 4 lives, replenish on win every 5 floors\n\
                        3. Defeat current champion for a reward"
                        }
                        GameMode::ArenaRanked => {
                            "1. Start with own team\n\
                        2. Defeat as many enemies as possible\n\
                        3. 4 lives, replenish on win every 5 floors\n\
                        4. Defeat current champion for a reward\n\
                        5. Credits reward depending on win streak\n\
                        6. No fee once a day"
                        }
                        GameMode::ArenaConst(_) => {
                            "1. Defeat as many enemies as possible\n\
                        2. 4 lives, replenish on win every 5 floors\n\
                        3. Entry fee growing every time, reset on day start\n\
                        4. Credits reward depending on win streak\n\
                        5. Fixed seed, everyone gets same units in shop\n\
                        6. Defeat current champion for a reward\n\
                        7. Rewards are multiplied by 2"
                        }
                    }
                    .cstr_cs(VISIBLE_LIGHT, CstrStyle::Small)
                    .as_label(ui)
                    .wrap()
                    .ui(ui);
                });
            });
        })
        .stretch_part(0.2)
        .pinned()
        .push(world);
    }
}
