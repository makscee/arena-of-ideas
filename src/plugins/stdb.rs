use serde_json::to_string_pretty;
use spacetimedb_lib::{de::serde::DeserializeWrapper, ser::serde::SerializeWrapper};
use spacetimedb_sdk::DbContext;

use crate::login;

use super::*;

#[derive(EnumIter, EnumString, AsRefStr, Hash, PartialEq, Eq, Display, Copy, Clone, Debug)]
pub enum StdbTable {
    GlobalSettings,
    GlobalData,

    TBaseUnit,
    THouse,
    TAbility,
    TStatus,

    TMetaShop,

    TTrade,

    TPlayer,
    TQuest,
    TArenaRun,
    TArenaRunArchive,
    TArenaLeaderboard,
    TTeam,
    TBattle,
    TAuction,
    TUnitItem,
    TUnitShardItem,
    TRainbowShardItem,
    TLootboxItem,
    TWallet,
    TDailyState,
    TUnitBalance,
    TIncubator,
    TIncubatorVote,
    TIncubatorFavorite,
    TPlayerStats,
    TPlayerGameStats,
}

#[derive(Clone, PartialEq, Eq, Debug)]
enum StdbCondition {
    Full,
    Owner,
    OwnerOrZero,
    Owners(Vec<u64>),
    OwnerMacro(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StdbQuery {
    table: StdbTable,
    condition: StdbCondition,
}

static SUBSCRIBED: OnceCell<Mutex<HashMap<StdbTable, StdbCondition>>> = OnceCell::new();
fn subscribed() -> MutexGuard<'static, HashMap<StdbTable, StdbCondition>> {
    SUBSCRIBED.get_or_init(|| default()).lock().unwrap()
}

impl ToCstr for StdbQuery {
    fn cstr(&self) -> Cstr {
        self.table
            .cstr()
            .push_wrapped_circ(self.condition.cstr())
            .take()
    }
}
impl ToCstr for StdbTable {
    fn cstr(&self) -> Cstr {
        self.as_ref().cstr_c(CYAN)
    }
}
impl ToCstr for StdbCondition {
    fn cstr(&self) -> Cstr {
        match self {
            StdbCondition::Full => "full".cstr_c(YELLOW),
            StdbCondition::Owner => "owner".cstr_c(GREEN),
            StdbCondition::OwnerOrZero => "owner or 0".cstr_c(GREEN),
            StdbCondition::Owners(l) => {
                format!("owners [{}]", l.into_iter().join(", ")).cstr_c(RED)
            }
            StdbCondition::OwnerMacro(q) => format!("owner macro {q}",).cstr_c(RED),
        }
    }
}

impl StdbQuery {
    pub fn is_subscribed(&self) -> bool {
        if let Some(c) = subscribed().get(&self.table) {
            match c {
                StdbCondition::Full => true,
                _ => self.condition.eq(c),
            }
        } else {
            false
        }
    }
    pub fn queries_login() -> Vec<StdbQuery> {
        [StdbTable::TPlayer.full(), StdbTable::GlobalData.full()].into()
    }
    pub fn queries_game() -> Vec<StdbQuery> {
        StdbTable::iter().map(|t| t.owner()).collect_vec()
    }
    fn query(self) -> String {
        let table = self.table.as_ref();
        let mut q = format!("select * from {table} ");
        let uid = player_id();
        match self.condition {
            StdbCondition::Full => {}
            StdbCondition::Owner => q.push_str(&format!("where owner = {uid}")),
            StdbCondition::OwnerOrZero => q.push_str(&format!("where owner = {uid} or owner = 0")),
            StdbCondition::Owners(l) => {
                let conditions = l.into_iter().map(|o| format!("owner = {o}")).join(" or ");
                q.push_str(&format!("where {conditions}"));
            }
            StdbCondition::OwnerMacro(m) => {
                let m = m.replace("{uid}", &player_id().to_string());
                q.push_str(&format!("where {m}"));
            }
        };
        q
    }
    pub fn subscribe(
        queries: impl IntoIterator<Item = StdbQuery>,
        on_subscribe: impl FnOnce(&mut World) + 'static + Send + Sync + Clone,
    ) {
        let mut subs = subscribed();
        for StdbQuery { table, condition } in queries {
            if let Some(replaced) = subs.insert(table, condition.clone()) {
                if replaced != condition {
                    debug!(
                        "{} on {table} from {} to {}",
                        "Subscription replaced".cyan(),
                        replaced.cstr(),
                        condition.cstr(),
                    );
                }
            }
        }

        let queries: Vec<Box<str>> = subs
            .iter()
            .map(|(t, c)| {
                StdbQuery {
                    table: *t,
                    condition: c.clone(),
                }
                .query()
                .into()
            })
            .collect_vec();
        info!("Update subscriptions:\n{}", queries.iter().join("\n"));
        cn().subscription_builder()
            .on_applied(move |e| {
                e.event.on_success(|world| {
                    on_subscribe(world);
                });
            })
            .subscribe(queries.into_boxed_slice());
    }
}

impl StdbTable {
    pub fn fill_from_json_data(self, json: &str, data: &mut GameData) {
        match self {
            StdbTable::GlobalSettings => {
                data.global_settings =
                    serde_json::from_str::<DeserializeWrapper<Vec<GlobalSettings>>>(json)
                        .unwrap()
                        .0;
            }
            StdbTable::GlobalData => {
                data.global_data =
                    serde_json::from_str::<DeserializeWrapper<Vec<GlobalData>>>(json)
                        .unwrap()
                        .0;
            }
            StdbTable::TBaseUnit => {
                data.base_unit = serde_json::from_str::<DeserializeWrapper<Vec<TBaseUnit>>>(json)
                    .unwrap()
                    .0;
            }
            StdbTable::THouse => {
                data.house = serde_json::from_str::<DeserializeWrapper<Vec<THouse>>>(json)
                    .unwrap()
                    .0;
            }
            StdbTable::TAbility => {
                data.ability = serde_json::from_str::<DeserializeWrapper<Vec<TAbility>>>(json)
                    .unwrap()
                    .0;
            }
            StdbTable::TStatus => {
                data.status = serde_json::from_str::<DeserializeWrapper<Vec<TStatus>>>(json)
                    .unwrap()
                    .0;
            }
            StdbTable::TMetaShop => {
                data.meta_shop = serde_json::from_str::<DeserializeWrapper<Vec<TMetaShop>>>(json)
                    .unwrap()
                    .0;
            }
            StdbTable::TTrade => {
                data.trade = serde_json::from_str::<DeserializeWrapper<Vec<TTrade>>>(json)
                    .unwrap()
                    .0;
            }
            StdbTable::TPlayer => {
                data.player = serde_json::from_str::<DeserializeWrapper<Vec<TPlayer>>>(json)
                    .unwrap()
                    .0;
            }
            StdbTable::TQuest => {
                data.quest = serde_json::from_str::<DeserializeWrapper<Vec<TQuest>>>(json)
                    .unwrap()
                    .0;
            }
            StdbTable::TArenaRun => {
                data.arena_run = serde_json::from_str::<DeserializeWrapper<Vec<TArenaRun>>>(json)
                    .unwrap()
                    .0;
            }
            StdbTable::TArenaRunArchive => {
                data.arena_run_archive =
                    serde_json::from_str::<DeserializeWrapper<Vec<TArenaRunArchive>>>(json)
                        .unwrap()
                        .0;
            }
            StdbTable::TArenaLeaderboard => {
                data.arena_leaderboard =
                    serde_json::from_str::<DeserializeWrapper<Vec<TArenaLeaderboard>>>(json)
                        .unwrap()
                        .0;
            }
            StdbTable::TTeam => {
                data.team = serde_json::from_str::<DeserializeWrapper<Vec<TTeam>>>(json)
                    .unwrap()
                    .0;
            }
            StdbTable::TBattle => {
                data.battle = serde_json::from_str::<DeserializeWrapper<Vec<TBattle>>>(json)
                    .unwrap()
                    .0;
            }
            StdbTable::TAuction => {
                data.auction = serde_json::from_str::<DeserializeWrapper<Vec<TAuction>>>(json)
                    .unwrap()
                    .0;
            }
            StdbTable::TUnitItem => {
                data.unit_item = serde_json::from_str::<DeserializeWrapper<Vec<TUnitItem>>>(json)
                    .unwrap()
                    .0;
            }
            StdbTable::TUnitShardItem => {
                data.unit_shard_item =
                    serde_json::from_str::<DeserializeWrapper<Vec<TUnitShardItem>>>(json)
                        .unwrap()
                        .0;
            }
            StdbTable::TRainbowShardItem => {
                data.rainbow_shard_item =
                    serde_json::from_str::<DeserializeWrapper<Vec<TRainbowShardItem>>>(json)
                        .unwrap()
                        .0;
            }
            StdbTable::TLootboxItem => {
                data.lootbox_item =
                    serde_json::from_str::<DeserializeWrapper<Vec<TLootboxItem>>>(json)
                        .unwrap()
                        .0;
            }
            StdbTable::TWallet => {
                data.wallet = serde_json::from_str::<DeserializeWrapper<Vec<TWallet>>>(json)
                    .unwrap()
                    .0;
            }
            StdbTable::TDailyState => {
                data.daily_state =
                    serde_json::from_str::<DeserializeWrapper<Vec<TDailyState>>>(json)
                        .unwrap()
                        .0;
            }
            StdbTable::TUnitBalance => {
                data.unit_balance =
                    serde_json::from_str::<DeserializeWrapper<Vec<TUnitBalance>>>(json)
                        .unwrap()
                        .0;
            }
            StdbTable::TIncubator => {
                data.incubator = serde_json::from_str::<DeserializeWrapper<Vec<TIncubator>>>(json)
                    .unwrap()
                    .0;
            }
            StdbTable::TIncubatorVote => {
                data.incubator_vote =
                    serde_json::from_str::<DeserializeWrapper<Vec<TIncubatorVote>>>(json)
                        .unwrap()
                        .0;
            }
            StdbTable::TIncubatorFavorite => {
                data.incubator_favorite =
                    serde_json::from_str::<DeserializeWrapper<Vec<TIncubatorFavorite>>>(json)
                        .unwrap()
                        .0;
            }
            StdbTable::TPlayerStats => {
                data.player_stats =
                    serde_json::from_str::<DeserializeWrapper<Vec<TPlayerStats>>>(json)
                        .unwrap()
                        .0;
            }
            StdbTable::TPlayerGameStats => {
                data.player_game_stats =
                    serde_json::from_str::<DeserializeWrapper<Vec<TPlayerGameStats>>>(json)
                        .unwrap()
                        .0;
            }
        }
    }
    pub fn get_json_data(self) -> String {
        match self {
            StdbTable::GlobalSettings => to_string_pretty(&SerializeWrapper::new(
                cn().db.global_settings().iter().collect_vec(),
            )),
            StdbTable::GlobalData => to_string_pretty(&SerializeWrapper::new(
                cn().db.global_data().iter().collect_vec(),
            )),
            StdbTable::TBaseUnit => to_string_pretty(&SerializeWrapper::new(
                cn().db.base_unit().iter().collect_vec(),
            )),
            StdbTable::THouse => {
                to_string_pretty(&SerializeWrapper::new(cn().db.house().iter().collect_vec()))
            }
            StdbTable::TAbility => to_string_pretty(&SerializeWrapper::new(
                cn().db.ability().iter().collect_vec(),
            )),
            StdbTable::TStatus => to_string_pretty(&SerializeWrapper::new(
                cn().db.status().iter().collect_vec(),
            )),
            StdbTable::TMetaShop => to_string_pretty(&SerializeWrapper::new(
                cn().db.meta_shop().iter().collect_vec(),
            )),
            StdbTable::TTrade => {
                to_string_pretty(&SerializeWrapper::new(cn().db.trade().iter().collect_vec()))
            }
            StdbTable::TPlayer => to_string_pretty(&SerializeWrapper::new(
                cn().db.player().iter().collect_vec(),
            )),
            StdbTable::TQuest => {
                to_string_pretty(&SerializeWrapper::new(cn().db.quest().iter().collect_vec()))
            }
            StdbTable::TArenaRun => to_string_pretty(&SerializeWrapper::new(
                cn().db.arena_run().iter().collect_vec(),
            )),
            StdbTable::TArenaRunArchive => to_string_pretty(&SerializeWrapper::new(
                cn().db.arena_run_archive().iter().collect_vec(),
            )),
            StdbTable::TArenaLeaderboard => to_string_pretty(&SerializeWrapper::new(
                cn().db.arena_leaderboard().iter().collect_vec(),
            )),
            StdbTable::TTeam => {
                to_string_pretty(&SerializeWrapper::new(cn().db.team().iter().collect_vec()))
            }
            StdbTable::TBattle => to_string_pretty(&SerializeWrapper::new(
                cn().db.battle().iter().collect_vec(),
            )),
            StdbTable::TAuction => to_string_pretty(&SerializeWrapper::new(
                cn().db.auction().iter().collect_vec(),
            )),
            StdbTable::TUnitItem => to_string_pretty(&SerializeWrapper::new(
                cn().db.unit_item().iter().collect_vec(),
            )),
            StdbTable::TUnitShardItem => to_string_pretty(&SerializeWrapper::new(
                cn().db.unit_shard_item().iter().collect_vec(),
            )),
            StdbTable::TRainbowShardItem => to_string_pretty(&SerializeWrapper::new(
                cn().db.rainbow_shard_item().iter().collect_vec(),
            )),
            StdbTable::TLootboxItem => to_string_pretty(&SerializeWrapper::new(
                cn().db.lootbox_item().iter().collect_vec(),
            )),
            StdbTable::TWallet => to_string_pretty(&SerializeWrapper::new(
                cn().db.wallet().iter().collect_vec(),
            )),
            StdbTable::TDailyState => to_string_pretty(&SerializeWrapper::new(
                cn().db.daily_state().iter().collect_vec(),
            )),
            StdbTable::TUnitBalance => to_string_pretty(&SerializeWrapper::new(
                cn().db.unit_balance().iter().collect_vec(),
            )),
            StdbTable::TIncubator => to_string_pretty(&SerializeWrapper::new(
                cn().db.incubator().iter().collect_vec(),
            )),
            StdbTable::TIncubatorVote => to_string_pretty(&SerializeWrapper::new(
                cn().db.incubator_vote().iter().collect_vec(),
            )),
            StdbTable::TIncubatorFavorite => to_string_pretty(&SerializeWrapper::new(
                cn().db.incubator_favorite().iter().collect_vec(),
            )),
            StdbTable::TPlayerStats => to_string_pretty(&SerializeWrapper::new(
                cn().db.player_stats().iter().collect_vec(),
            )),
            StdbTable::TPlayerGameStats => to_string_pretty(&SerializeWrapper::new(
                cn().db.player_game_stats().iter().collect_vec(),
            )),
        }
        .unwrap()
    }
    pub fn full(self) -> StdbQuery {
        StdbQuery {
            table: self,
            condition: StdbCondition::Full,
        }
    }
    pub fn owner(self) -> StdbQuery {
        match self {
            StdbTable::GlobalSettings
            | StdbTable::GlobalData
            | StdbTable::TBaseUnit
            | StdbTable::THouse
            | StdbTable::TAbility
            | StdbTable::TStatus
            | StdbTable::TArenaLeaderboard
            | StdbTable::TBattle
            | StdbTable::TAuction
            | StdbTable::TTeam
            | StdbTable::TPlayer
            | StdbTable::TArenaRunArchive
            | StdbTable::TIncubator
            | StdbTable::TIncubatorVote
            | StdbTable::TIncubatorFavorite
            | StdbTable::TPlayerStats
            | StdbTable::TPlayerGameStats
            | StdbTable::TMetaShop => self.full(),

            StdbTable::TTrade => StdbQuery {
                table: self,
                condition: StdbCondition::OwnerMacro("a_player = {uid} or b_player = {uid}".into()),
            },

            StdbTable::TUnitItem | StdbTable::TQuest => StdbQuery {
                table: self,
                condition: StdbCondition::OwnerOrZero,
            },
            StdbTable::TUnitShardItem | StdbTable::TRainbowShardItem | StdbTable::TLootboxItem => {
                StdbQuery {
                    table: self,
                    condition: StdbCondition::OwnerMacro(
                        "(owner = {uid} or owner = 0) and count > 0".into(),
                    ),
                }
            }

            StdbTable::TArenaRun
            | StdbTable::TWallet
            | StdbTable::TDailyState
            | StdbTable::TUnitBalance => StdbQuery {
                table: self,
                condition: StdbCondition::Owner,
            },
        }
    }
}

pub fn apply_subscriptions(dbc: &DbConnection) {
    let r = dbc.reducers();
    r.on_incubator_post(|e, u| {
        let unit = u.name.clone();
        e.event.on_success(move |world| {
            Notification::new(
                format!("Unit {} submitted to Incubator", unit).cstr_c(VISIBLE_LIGHT),
            )
            .push(world);
        });
    });
    r.on_incubator_update(|e, id, u| {
        let unit = u.name.clone();
        e.event.on_success(move |world| {
            Notification::new(format!("Unit {} updated in Incubator", unit).cstr_c(VISIBLE_LIGHT))
                .push(world);
        });
    });
    r.on_run_start_normal(|e| e.event.on_success(|w| GameState::Shop.proceed_to_target(w)));
    r.on_run_start_ranked(|e, _| e.event.on_success(|w| GameState::Shop.proceed_to_target(w)));
    r.on_run_start_const(|e| e.event.on_success(|w| GameState::Shop.proceed_to_target(w)));

    r.on_incubator_delete(|e, id| {
        let id = *id;
        e.event.on_success(move |world| {
            TilePlugin::close(&IncubatorPlugin::tile_id(id), world);
            Notification::new_string(format!("Incubator entry#{id} deleted")).push(world)
        });
    });

    r.on_login_by_identity(|e| {
        e.event.on_success(|world| {
            LoginPlugin::complete(None, world);
        });
    });
    r.on_register_empty(|e| {
        e.event.on_success(|world| {
            Notification::new_string("New player created".to_owned()).push(world);
            let identity = ConnectOption::get(world).identity.clone();
            let player = cn()
                .db
                .player()
                .iter()
                .find(|u| u.identities.contains(&identity))
                .expect("Failed to find player after registration");
            world.resource_mut::<LoginData>().identity_player = Some(player);
        })
    });
    r.on_login(|e, name, pass| {
        let name = name.clone();
        let player = e.db.player().name().find(&name).unwrap();
        e.event.on_success(move |world| {
            LoginPlugin::complete(Some(player), world);
        })
    });

    r.on_auction_create(|e, _, _, _| e.event.on_success(|w| "Auction created".notify(w)));
    r.on_auction_buy(|e, id| {
        let id = *id;
        e.event
            .on_success(move |world| format!("Auction#{id} bought").notify(world));
    });

    r.on_unit_balance_vote(|e, unit, vote| {
        let unit = unit.clone();
        let vote = if *vote >= 0 {
            format!("+{vote}")
        } else {
            vote.to_string()
        };
        e.event.on_success(move |w| {
            format!("Vote accepted: {unit} {vote}").notify(w);
            MetaPlugin::get_next_for_balancing(w);
        });
    });

    r.on_dismantle_hero(|e, _| e.event.notify_error());
    r.on_craft_hero(|e, u, i| {
        let unit = u.clone();
        e.event.on_success(move |world| {
            format!("{unit} crafted").notify(world);
        })
    });
    r.on_open_lootbox(|e, i| {
        e.event.on_success(|world| "Lootbox opened".notify(world));
    });

    r.on_set_name(|e, name| {
        let name = name.clone();
        e.event.on_success(move |world| {
            format!("Name successfully changed to {name}").notify(world);
            ProfilePlugin::update_user(world);
        })
    });

    r.on_set_password(|e, _, _| {
        e.event.on_success(|world| {
            "Password updated successfully".notify(world);
            ProfilePlugin::update_user(world);
            ProfilePlugin::clear_edit(world);
        })
    });

    r.on_shop_finish(|e, _| {
        e.event
            .on_success(|w| GameState::ShopBattle.proceed_to_target(w))
    });

    r.on_stack_team(|e, _, _| e.event.notify_error());
    r.on_stack_shop(|e, _, _| e.event.notify_error());
    r.on_fuse_start(|e, _, _| e.event.notify_error());

    r.on_run_finish(|e| {
        e.event.on_success(|w| {
            GameState::GameStart.proceed_to_target(w);
        });
    });

    r.on_team_create(|e, _| {
        e.event
            .on_success(|world| TeamPlugin::load_teams_table(world));
    });
    r.on_team_rename(|e, _, _| {
        e.event
            .on_success(|world| TeamPlugin::load_teams_table(world));
    });

    r.on_team_disband(|e, id| {
        let id = *id;
        e.event.on_success(move |world| {
            format!("Team#{id} disbanded").notify(world);
            MetaPlugin::clear(world);
            MetaPlugin::load_mode(MetaMode::Teams, world);
        })
    });
    r.on_team_add_unit(|e, _, _| {
        e.event.on_success(|world| {
            TeamPlugin::load_teams_table(world);
            Confirmation::close_current(world);
        })
    });
    r.on_team_remove_unit(|e, _, _| {
        e.event.on_success(|w| {
            "Unit removed".notify(w);
            TeamPlugin::load_teams_table(w);
        });
    });
    r.on_team_swap_units(|e, _, _, _| e.event.notify_error());

    r.on_meta_buy(|e, _| {
        e.event.notify_error();
    });

    r.on_accept_trade(|e, _| {
        e.event.notify_error();
    });
}
