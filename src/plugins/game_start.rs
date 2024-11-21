use super::*;

pub struct GameStartPlugin;

#[derive(Resource)]
struct GameStartResource {
    selected_mode: GameMode,
    selected_season: u32,
    teams: Vec<TTeam>,
    selected_team: usize,
    right_mode: Mode,
}

fn rm(world: &mut World) -> Mut<GameStartResource> {
    world.resource_mut::<GameStartResource>()
}

#[derive(Default, Clone, Copy, AsRefStr, PartialEq, Eq, EnumIter, Hash)]
enum Mode {
    #[default]
    Runs,
    Battles,
}

impl ToCstr for Mode {
    fn cstr(&self) -> Cstr {
        self.as_ref().cstr_c(VISIBLE_LIGHT)
    }
}

impl Default for GameStartResource {
    fn default() -> Self {
        Self {
            selected_mode: client_state().last_played_mode.unwrap_or_default().into(),
            teams: default(),
            selected_team: 0,
            selected_season: global_settings().season,
            right_mode: default(),
        }
    }
}

impl GameStartPlugin {
    fn load_leaderboard(mode: GameMode, season: u32) -> Vec<TArenaLeaderboard> {
        cn().db
            .arena_leaderboard()
            .iter()
            .filter(|a| a.season == season && a.mode == mode)
            .sorted_by_key(|d| -(d.floor as i32))
            .collect()
    }
    fn load_battle_history(mode: GameMode) -> Vec<TBattle> {
        cn().db
            .battle()
            .iter()
            .filter(|b| b.mode == mode)
            .sorted_by_key(|d| -(d.id as i32))
            .collect()
    }
    fn load_arena_runs(mode: GameMode, season: u32) -> Vec<TArenaRunArchive> {
        cn().db
            .arena_run_archive()
            .iter()
            .filter(|d| d.season == season && d.mode == mode)
            .sorted_by_key(|d| -(d.id as i32))
            .collect()
    }
    fn load_data(world: &mut World) {
        TableState::reset_cache(&egui_context(world).unwrap());
        let mut gsr = rm(world);
        gsr.teams = cn()
            .db
            .team()
            .iter()
            .filter(|t| t.owner == player_id() && t.pool == TeamPool::Owned && !t.units.is_empty())
            .collect_vec();
        if let Some(i) = client_state()
            .last_played_team
            .and_then(|id| gsr.teams.iter().position(|t| t.id == id))
        {
            gsr.selected_team = i;
        }
    }
    fn current_champion(mode: GameMode) -> Option<TArenaLeaderboard> {
        cn().db
            .arena_leaderboard()
            .iter()
            .filter(|d| d.mode == mode)
            .max_by_key(|d| d.floor)
    }
    pub fn add_tiles(world: &mut World) {
        if !world.is_resource_added::<GameStartResource>() {
            world.init_resource::<GameStartResource>();
        }
        Self::load_data(world);
        Tile::new(Side::Left, Self::show_middle)
            .stretch_min()
            .pinned()
            .push(world);
        Tile::new(Side::Left, |ui, world| {
            let mut r = rm(world);
            if season_switcher(&mut r.selected_season, ui) {
                Self::load_data(world);
                return;
            }
            title("Leaderboard", ui);
            if let Some(champion) = Self::current_champion(r.selected_mode) {
                ui.vertical_centered_justified(|ui| {
                    "Current champion"
                        .cstr_cs(YELLOW, CstrStyle::Bold)
                        .label(ui);
                    champion
                        .owner
                        .get_player()
                        .name
                        .cstr_cs(VISIBLE_BRIGHT, CstrStyle::Heading2)
                        .label(ui);
                });
            }
            ScrollArea::both().auto_shrink(false).show(ui, |ui| {
                Table::new("Leaderboard", |world| {
                    let gsr = rm(world);
                    Self::load_leaderboard(gsr.selected_mode, gsr.selected_season)
                })
                .add_arena_leaderboard_columns(|d| d)
                .ui(ui, world);
            });
        })
        .pinned()
        .transparent()
        .stretch_part(0.5)
        .push_front(world);
        Tile::new(Side::Right, |ui, world| {
            Self::show_right(ui, world);
        })
        .pinned()
        .transparent()
        .stretch_max()
        .push(world);
    }
    fn show_middle(ui: &mut Ui, world: &mut World) {
        ui.set_width(250.0);
        if game_mode_switcher(&mut rm(world).selected_mode, ui) {
            Self::load_data(world);
            return;
        }
        br(ui);
        ui.vertical_centered_justified(|ui| {
            let r = rm(world);
            let mode = r.selected_mode.clone();
            mode.cstr_s(CstrStyle::Heading).label(ui);
            ui.add_space(30.0);
            let run = cn().db.arena_run().get_current();
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
                            let cost = cn().db.daily_state().current().ranked_cost;
                            let mut new_selected = None;
                            ui.horizontal_wrapped(|ui| {
                                for (i, team) in r.teams.iter().enumerate() {
                                    if Button::new(team.cstr())
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
                    GameMode::ArenaConst => {
                        let cost = cn().db.daily_state().current().const_cost;
                        if cost == 0 {
                            "First daily run free!"
                                .cstr_cs(GREEN, CstrStyle::Bold)
                                .label(ui);
                            space(ui);
                        }
                        entry_fee = Some(cost);
                    }
                };
                let mut btn = Button::new("Play".cstr_s(CstrStyle::Heading));
                if let Some(cost) = entry_fee {
                    btn = btn.credits_cost(cost);
                    enabled = enabled && can_afford(cost);
                }
                if btn.enabled(enabled).ui(ui).clicked() {
                    let r = rm(world);
                    let mut cs = client_state().clone();
                    cs.last_played_mode = Some(r.selected_mode.clone().into());
                    match &r.selected_mode {
                        GameMode::ArenaNormal => {
                            let _ = cn().reducers.run_start_normal();
                        }
                        GameMode::ArenaRanked => {
                            let team = r.teams[r.selected_team].id;
                            cs.last_played_team = Some(team);
                            let _ = cn().reducers.run_start_ranked(team);
                        }
                        GameMode::ArenaConst => {
                            let _ = cn().reducers.run_start_const();
                        }
                    }
                    cs.save();
                }
            }
            ui.add_space(13.0);
            if let Some(run) = run {
                ui.vertical_centered(|ui| {
                    if Button::new("Continue".cstr_cs(VISIBLE_BRIGHT, CstrStyle::Heading))
                        .ui(ui)
                        .clicked()
                    {
                        GameState::Shop.proceed_to_target(world);
                    }
                    if Button::new("Abandon run").red(ui).ui(ui).clicked() {
                        Confirmation::new("Abandon current run?")
                            .accept(|_| {
                                cn().reducers.run_finish().unwrap();
                            })
                            .cancel(|_| {})
                            .push(world);
                    }
                    ui.add_space(20.0);
                    ShopPlugin::show_stats(&run, ui);
                });
            }
            ui.with_layout(Layout::bottom_up(Align::LEFT), |ui| {
                match &mode {
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
                    GameMode::ArenaConst => {
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
    }
    fn show_right(ui: &mut Ui, world: &mut World) {
        EnumSwitcher::new().show(&mut rm(world).right_mode, ui);
        let right_mode = rm(world).right_mode;
        match right_mode {
            Mode::Runs => {
                title("Runs", ui);
                ScrollArea::both().auto_shrink(false).show(ui, |ui| {
                    Table::new("Runs", |world| {
                        let gsr = rm(world);
                        Self::load_arena_runs(gsr.selected_mode, gsr.selected_season)
                    })
                    .add_arena_run_archive_columns(|d| d)
                    .ui(ui, world)
                });
            }
            Mode::Battles => {
                title("Battles", ui);
                ScrollArea::both().auto_shrink(false).show(ui, |ui| {
                    ui.push_id(right_mode, |ui| {
                        Table::new("Battle History", |world| {
                            let gsr = rm(world);
                            Self::load_battle_history(gsr.selected_mode)
                        })
                        .column_ts("time", |d: &TBattle| d.ts)
                        .column_cstr("result", |d, _| match d.result {
                            TBattleResult::Tbd => "-".cstr(),
                            TBattleResult::Left => "W".cstr_c(GREEN),
                            TBattleResult::Right | TBattleResult::Even => "L".cstr_c(RED),
                        })
                        .column_player_click("player", |d| d.owner)
                        .column_team("player >", |d| d.team_left)
                        .column_team("< enemy", |d| d.team_right)
                        .column_player_click("enemy", |d| d.team_right.get_team().owner)
                        .column_id("id", |d| d.id)
                        .column_cstr("mode", |d, _| d.mode.cstr())
                        .column_btn("copy", |d, _, world| {
                            copy_to_clipboard(
                                &ron::to_string(&BattleResource::from(d.clone())).unwrap(),
                                world,
                            );
                        })
                        .column_btn("editor", |d, _, world| {
                            EditorPlugin::load_battle(
                                PackedTeam::from_id(d.team_left),
                                PackedTeam::from_id(d.team_right),
                                world,
                            );
                            GameState::Editor.set_next(world);
                        })
                        .column_btn("run", |d, _, world| {
                            world.insert_resource(BattleResource::from(d.clone()));
                            BattlePlugin::set_next_state(cur_state(world), world);
                            GameState::Battle.set_next(world);
                        })
                        .filter("My", "player", player_id().into())
                        .filter("Win", "result", "W".into())
                        .filter("Lose", "result", "L".into())
                        .filter("TBD", "result", "-".into())
                        .ui(ui, world)
                    });
                });
            }
        }
    }
}
