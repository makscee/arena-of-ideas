use serde_json::to_string_pretty;
use spacetimedb_lib::{de::serde::DeserializeWrapper, ser::serde::SerializeWrapper};
use spacetimedb_sdk::{DbContext, TableWithPrimaryKey};

use crate::login;

use super::*;

#[derive(EnumIter, EnumString, AsRefStr, Hash, PartialEq, Eq, Display, Copy, Clone, Debug)]
#[allow(non_camel_case_types)]
pub enum StdbTable {
    global_settings,
    global_data,

    base_unit,
    house,
    ability,
    status,

    meta_shop,

    trade,

    player,
    quest,
    arena_run,
    arena_run_archive,
    arena_leaderboard,
    team,
    battle,
    auction,
    unit_item,
    unit_shard_item,
    rainbow_shard_item,
    lootbox_item,
    wallet,
    daily_state,
    unit_balance,
    incubator,
    incubator_vote,
    incubator_favorite,
    player_stats,
    player_game_stats,
    global_event,
    player_tag,
    reward,
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
        [StdbTable::player.full(), StdbTable::global_data.full()].into()
    }
    pub fn queries_game() -> Vec<StdbQuery> {
        StdbTable::iter().filter_map(|t| t.owner()).collect_vec()
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
            .on_error(|e| e.event.notify_error())
            .on_applied(move |e| {
                info!("Subscription applied");
                e.event.on_success(|world| {
                    info!("Subscription applied");
                    on_subscribe(world);
                });
            })
            .subscribe(queries.into_boxed_slice());
    }
}

impl StdbTable {
    pub fn fill_from_json_data(self, json: &str, data: &mut GameData) {
        match self {
            StdbTable::global_settings => {
                data.global_settings =
                    serde_json::from_str::<DeserializeWrapper<Vec<GlobalSettings>>>(json)
                        .unwrap()
                        .0;
            }
            StdbTable::global_data => {
                data.global_data =
                    serde_json::from_str::<DeserializeWrapper<Vec<GlobalData>>>(json)
                        .unwrap()
                        .0;
            }
            StdbTable::base_unit => {
                data.base_unit = serde_json::from_str::<DeserializeWrapper<Vec<TBaseUnit>>>(json)
                    .unwrap()
                    .0;
            }
            StdbTable::house => {
                data.house = serde_json::from_str::<DeserializeWrapper<Vec<THouse>>>(json)
                    .unwrap()
                    .0;
            }
            StdbTable::ability => {
                data.ability = serde_json::from_str::<DeserializeWrapper<Vec<TAbility>>>(json)
                    .unwrap()
                    .0;
            }
            StdbTable::status => {
                data.status = serde_json::from_str::<DeserializeWrapper<Vec<TStatus>>>(json)
                    .unwrap()
                    .0;
            }
            StdbTable::meta_shop => {
                data.meta_shop = serde_json::from_str::<DeserializeWrapper<Vec<TMetaShop>>>(json)
                    .unwrap()
                    .0;
            }
            StdbTable::trade => {
                data.trade = serde_json::from_str::<DeserializeWrapper<Vec<TTrade>>>(json)
                    .unwrap()
                    .0;
            }
            StdbTable::player => {
                data.player = serde_json::from_str::<DeserializeWrapper<Vec<TPlayer>>>(json)
                    .unwrap()
                    .0;
            }
            StdbTable::quest => {
                data.quest = serde_json::from_str::<DeserializeWrapper<Vec<TQuest>>>(json)
                    .unwrap()
                    .0;
            }
            StdbTable::arena_run => {
                data.arena_run = serde_json::from_str::<DeserializeWrapper<Vec<TArenaRun>>>(json)
                    .unwrap()
                    .0;
            }
            StdbTable::arena_run_archive => {
                data.arena_run_archive =
                    serde_json::from_str::<DeserializeWrapper<Vec<TArenaRunArchive>>>(json)
                        .unwrap()
                        .0;
            }
            StdbTable::arena_leaderboard => {
                data.arena_leaderboard =
                    serde_json::from_str::<DeserializeWrapper<Vec<TArenaLeaderboard>>>(json)
                        .unwrap()
                        .0;
            }
            StdbTable::team => {
                data.team = serde_json::from_str::<DeserializeWrapper<Vec<TTeam>>>(json)
                    .unwrap()
                    .0;
            }
            StdbTable::battle => {
                data.battle = serde_json::from_str::<DeserializeWrapper<Vec<TBattle>>>(json)
                    .unwrap()
                    .0;
            }
            StdbTable::auction => {
                data.auction = serde_json::from_str::<DeserializeWrapper<Vec<TAuction>>>(json)
                    .unwrap()
                    .0;
            }
            StdbTable::unit_item => {
                data.unit_item = serde_json::from_str::<DeserializeWrapper<Vec<TUnitItem>>>(json)
                    .unwrap()
                    .0;
            }
            StdbTable::unit_shard_item => {
                data.unit_shard_item =
                    serde_json::from_str::<DeserializeWrapper<Vec<TUnitShardItem>>>(json)
                        .unwrap()
                        .0;
            }
            StdbTable::rainbow_shard_item => {
                data.rainbow_shard_item =
                    serde_json::from_str::<DeserializeWrapper<Vec<TRainbowShardItem>>>(json)
                        .unwrap()
                        .0;
            }
            StdbTable::lootbox_item => {
                data.lootbox_item =
                    serde_json::from_str::<DeserializeWrapper<Vec<TLootboxItem>>>(json)
                        .unwrap()
                        .0;
            }
            StdbTable::wallet => {
                data.wallet = serde_json::from_str::<DeserializeWrapper<Vec<TWallet>>>(json)
                    .unwrap()
                    .0;
            }
            StdbTable::daily_state => {
                data.daily_state =
                    serde_json::from_str::<DeserializeWrapper<Vec<TDailyState>>>(json)
                        .unwrap()
                        .0;
            }
            StdbTable::unit_balance => {
                data.unit_balance =
                    serde_json::from_str::<DeserializeWrapper<Vec<TUnitBalance>>>(json)
                        .unwrap()
                        .0;
            }
            StdbTable::incubator => {
                data.incubator = serde_json::from_str::<DeserializeWrapper<Vec<TIncubator>>>(json)
                    .unwrap()
                    .0;
            }
            StdbTable::incubator_vote => {
                data.incubator_vote =
                    serde_json::from_str::<DeserializeWrapper<Vec<TIncubatorVote>>>(json)
                        .unwrap()
                        .0;
            }
            StdbTable::incubator_favorite => {
                data.incubator_favorite =
                    serde_json::from_str::<DeserializeWrapper<Vec<TIncubatorFavorite>>>(json)
                        .unwrap()
                        .0;
            }
            StdbTable::player_stats => {
                data.player_stats =
                    serde_json::from_str::<DeserializeWrapper<Vec<TPlayerStats>>>(json)
                        .unwrap()
                        .0;
            }
            StdbTable::player_game_stats => {
                data.player_game_stats =
                    serde_json::from_str::<DeserializeWrapper<Vec<TPlayerGameStats>>>(json)
                        .unwrap()
                        .0;
            }
            StdbTable::global_event => {
                data.global_event =
                    serde_json::from_str::<DeserializeWrapper<Vec<TGlobalEvent>>>(json)
                        .unwrap()
                        .0;
            }
            StdbTable::player_tag => {
                data.player_tag = serde_json::from_str::<DeserializeWrapper<Vec<TPlayerTag>>>(json)
                    .unwrap()
                    .0;
            }
            StdbTable::reward => {
                data.reward = serde_json::from_str::<DeserializeWrapper<Vec<TReward>>>(json)
                    .unwrap()
                    .0;
            }
        }
    }
    pub fn get_json_data(self) -> String {
        match self {
            StdbTable::global_settings => to_string_pretty(&SerializeWrapper::new(
                cn().db.global_settings().iter().collect_vec(),
            )),
            StdbTable::global_data => to_string_pretty(&SerializeWrapper::new(
                cn().db.global_data().iter().collect_vec(),
            )),
            StdbTable::base_unit => to_string_pretty(&SerializeWrapper::new(
                cn().db.base_unit().iter().collect_vec(),
            )),
            StdbTable::house => {
                to_string_pretty(&SerializeWrapper::new(cn().db.house().iter().collect_vec()))
            }
            StdbTable::ability => to_string_pretty(&SerializeWrapper::new(
                cn().db.ability().iter().collect_vec(),
            )),
            StdbTable::status => to_string_pretty(&SerializeWrapper::new(
                cn().db.status().iter().collect_vec(),
            )),
            StdbTable::meta_shop => to_string_pretty(&SerializeWrapper::new(
                cn().db.meta_shop().iter().collect_vec(),
            )),
            StdbTable::trade => {
                to_string_pretty(&SerializeWrapper::new(cn().db.trade().iter().collect_vec()))
            }
            StdbTable::player => to_string_pretty(&SerializeWrapper::new(
                cn().db.player().iter().collect_vec(),
            )),
            StdbTable::quest => {
                to_string_pretty(&SerializeWrapper::new(cn().db.quest().iter().collect_vec()))
            }
            StdbTable::arena_run => to_string_pretty(&SerializeWrapper::new(
                cn().db.arena_run().iter().collect_vec(),
            )),
            StdbTable::arena_run_archive => to_string_pretty(&SerializeWrapper::new(
                cn().db.arena_run_archive().iter().collect_vec(),
            )),
            StdbTable::arena_leaderboard => to_string_pretty(&SerializeWrapper::new(
                cn().db.arena_leaderboard().iter().collect_vec(),
            )),
            StdbTable::team => {
                to_string_pretty(&SerializeWrapper::new(cn().db.team().iter().collect_vec()))
            }
            StdbTable::battle => to_string_pretty(&SerializeWrapper::new(
                cn().db.battle().iter().collect_vec(),
            )),
            StdbTable::auction => to_string_pretty(&SerializeWrapper::new(
                cn().db.auction().iter().collect_vec(),
            )),
            StdbTable::unit_item => to_string_pretty(&SerializeWrapper::new(
                cn().db.unit_item().iter().collect_vec(),
            )),
            StdbTable::unit_shard_item => to_string_pretty(&SerializeWrapper::new(
                cn().db.unit_shard_item().iter().collect_vec(),
            )),
            StdbTable::rainbow_shard_item => to_string_pretty(&SerializeWrapper::new(
                cn().db.rainbow_shard_item().iter().collect_vec(),
            )),
            StdbTable::lootbox_item => to_string_pretty(&SerializeWrapper::new(
                cn().db.lootbox_item().iter().collect_vec(),
            )),
            StdbTable::wallet => to_string_pretty(&SerializeWrapper::new(
                cn().db.wallet().iter().collect_vec(),
            )),
            StdbTable::daily_state => to_string_pretty(&SerializeWrapper::new(
                cn().db.daily_state().iter().collect_vec(),
            )),
            StdbTable::unit_balance => to_string_pretty(&SerializeWrapper::new(
                cn().db.unit_balance().iter().collect_vec(),
            )),
            StdbTable::incubator => to_string_pretty(&SerializeWrapper::new(
                cn().db.incubator().iter().collect_vec(),
            )),
            StdbTable::incubator_vote => to_string_pretty(&SerializeWrapper::new(
                cn().db.incubator_vote().iter().collect_vec(),
            )),
            StdbTable::incubator_favorite => to_string_pretty(&SerializeWrapper::new(
                cn().db.incubator_favorite().iter().collect_vec(),
            )),
            StdbTable::player_stats => to_string_pretty(&SerializeWrapper::new(
                cn().db.player_stats().iter().collect_vec(),
            )),
            StdbTable::player_game_stats => to_string_pretty(&SerializeWrapper::new(
                cn().db.player_game_stats().iter().collect_vec(),
            )),
            StdbTable::global_event => to_string_pretty(&SerializeWrapper::new(
                cn().db.global_event().iter().collect_vec(),
            )),
            StdbTable::player_tag => to_string_pretty(&SerializeWrapper::new(
                cn().db.player_tag().iter().collect_vec(),
            )),
            StdbTable::reward => to_string_pretty(&SerializeWrapper::new(
                cn().db.reward().iter().collect_vec(),
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
    pub fn owner(self) -> Option<StdbQuery> {
        match self {
            StdbTable::global_settings
            | StdbTable::global_data
            | StdbTable::base_unit
            | StdbTable::house
            | StdbTable::ability
            | StdbTable::status
            | StdbTable::arena_leaderboard
            | StdbTable::battle
            | StdbTable::auction
            | StdbTable::team
            | StdbTable::player
            | StdbTable::arena_run_archive
            | StdbTable::incubator
            | StdbTable::incubator_vote
            | StdbTable::incubator_favorite
            | StdbTable::player_stats
            | StdbTable::player_game_stats
            | StdbTable::meta_shop
            | StdbTable::player_tag => Some(self.full()),

            StdbTable::trade => Some(StdbQuery {
                table: self,
                condition: StdbCondition::OwnerMacro("a_player = {uid} or b_player = {uid}".into()),
            }),

            StdbTable::unit_item | StdbTable::quest => Some(StdbQuery {
                table: self,
                condition: StdbCondition::OwnerOrZero,
            }),
            StdbTable::unit_shard_item
            | StdbTable::rainbow_shard_item
            | StdbTable::lootbox_item => Some(StdbQuery {
                table: self,
                condition: StdbCondition::OwnerMacro(
                    "(owner = {uid} or owner = 0) and count > 0".into(),
                ),
            }),

            StdbTable::arena_run
            | StdbTable::wallet
            | StdbTable::daily_state
            | StdbTable::unit_balance
            | StdbTable::reward => Some(StdbQuery {
                table: self,
                condition: StdbCondition::Owner,
            }),

            StdbTable::global_event => None,
        }
    }
}

pub fn reducers_subscriptions(dbc: &DbConnection) {
    let r = dbc.reducers();
    r.on_incubator_post(|e, u| {
        if !e.check_identity() {
            return;
        }
        let unit = u.name.clone();
        e.event.on_success(move |world| {
            Notification::new(
                format!("Unit {} submitted to Incubator", unit).cstr_c(VISIBLE_LIGHT),
            )
            .push(world);
        });
    });
    r.on_incubator_update(|e, _, u| {
        if !e.check_identity() {
            return;
        }
        let unit = u.name.clone();
        e.event.on_success(move |world| {
            Notification::new(format!("Unit {} updated in Incubator", unit).cstr_c(VISIBLE_LIGHT))
                .push(world);
        });
    });
    r.on_run_start_normal(|e| {
        if !e.check_identity() {
            return;
        }
        e.event.on_success(|w| GameState::Shop.proceed_to_target(w))
    });
    r.on_run_start_ranked(|e, _| {
        if !e.check_identity() {
            return;
        }
        e.event.on_success(|w| GameState::Shop.proceed_to_target(w))
    });
    r.on_run_start_const(|e| {
        if !e.check_identity() {
            return;
        }
        e.event.on_success(|w| GameState::Shop.proceed_to_target(w))
    });

    r.on_incubator_delete(|e, id| {
        if !e.check_identity() {
            return;
        }
        let id = *id;
        e.event.on_success(move |world| {
            TilePlugin::close(&IncubatorPlugin::tile_id(id), world);
            Notification::new_string(format!("Incubator entry#{id} deleted")).push(world)
        });
    });

    r.on_login_by_identity(|e| {
        if !e.check_identity() {
            return;
        }
        e.event.on_success(|world| {
            LoginPlugin::complete(None, world);
        });
    });
    r.on_register_empty(|e| {
        if !e.check_identity() {
            return;
        }
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
    r.on_login(|e, name, _| {
        if !e.check_identity() {
            return;
        }
        let name = name.clone();
        let player = e.db.player().name().find(&name).unwrap();
        e.event.on_success(move |world| {
            LoginPlugin::complete(Some(player), world);
        })
    });

    r.on_auction_create(|e, _, _, _| {
        if !e.check_identity() {
            return;
        }
        e.event.on_success(|w| "Auction created".notify(w))
    });
    r.on_auction_buy(|e, id| {
        if !e.check_identity() {
            return;
        }
        let id = *id;
        e.event
            .on_success(move |world| format!("Auction#{id} bought").notify(world));
    });

    r.on_unit_balance_vote(|e, unit, vote| {
        if !e.check_identity() {
            return;
        }
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

    r.on_dismantle_hero(|e, _| {
        if !e.check_identity() {
            return;
        }
        e.event.notify_error()
    });
    r.on_craft_hero(|e, u, _| {
        if !e.check_identity() {
            return;
        }
        let unit = u.clone();
        e.event.on_success(move |world| {
            format!("{unit} crafted").notify(world);
        })
    });
    r.on_open_lootbox(|e, _| {
        if !e.check_identity() {
            return;
        }
        e.event.on_success(|world| "Lootbox opened".notify(world));
    });

    r.on_set_name(|e, name| {
        if !e.check_identity() {
            return;
        }
        let name = name.clone();
        e.event.on_success(move |world| {
            format!("Name successfully changed to {name}").notify(world);
            ProfilePlugin::update_user(world);
        })
    });

    r.on_set_password(|e, _, _| {
        if !e.check_identity() {
            return;
        }
        e.event.on_success(|world| {
            "Password updated successfully".notify(world);
            ProfilePlugin::update_user(world);
            ProfilePlugin::clear_edit(world);
        })
    });

    r.on_shop_finish(|e, _| {
        if !e.check_identity() {
            return;
        }
        e.event
            .on_success(|w| GameState::ShopBattle.proceed_to_target(w))
    });

    r.on_stack_team(|e, _, _| {
        if !e.check_identity() {
            return;
        }
        e.event.notify_error()
    });
    r.on_stack_shop(|e, _, _| {
        if !e.check_identity() {
            return;
        }
        e.event.notify_error()
    });
    r.on_fuse_start(|e, _, _| {
        if !e.check_identity() {
            return;
        }
        e.event.notify_error()
    });

    r.on_run_finish(|e| {
        if !e.check_identity() {
            return;
        }
        e.event.on_success(|w| {
            GameState::GameStart.proceed_to_target(w);
        });
    });

    r.on_team_create(|e, _| {
        if !e.check_identity() {
            return;
        }
        e.event
            .on_success(|world| TeamPlugin::load_teams_table(world));
    });
    r.on_team_rename(|e, _, _| {
        if !e.check_identity() {
            return;
        }
        e.event
            .on_success(|world| TeamPlugin::load_teams_table(world));
    });

    r.on_team_disband(|e, id| {
        if !e.check_identity() {
            return;
        }
        let id = *id;
        e.event.on_success(move |world| {
            format!("Team#{id} disbanded").notify(world);
            MetaPlugin::clear(world);
            MetaPlugin::load_mode(MetaMode::Teams, world);
        })
    });
    r.on_team_add_unit(|e, _, _| {
        if !e.check_identity() {
            return;
        }
        e.event.on_success(|world| {
            TeamPlugin::load_teams_table(world);
            Confirmation::close_current(world);
        })
    });
    r.on_team_remove_unit(|e, _, _| {
        if !e.check_identity() {
            return;
        }
        e.event.on_success(|w| {
            "Unit removed".notify(w);
            TeamPlugin::load_teams_table(w);
        });
    });
    r.on_team_swap_units(|e, _, _, _| {
        if !e.check_identity() {
            return;
        }
        e.event.notify_error()
    });

    r.on_meta_buy(|e, _| {
        if !e.check_identity() {
            return;
        }
        e.event.notify_error();
    });

    r.on_accept_trade(|e, _| {
        if !e.check_identity() {
            return;
        }
        e.event.notify_error();
    });
}

pub fn db_subscriptions() {
    let db = &cn().db;
    db.trade().on_insert(|e, r| {
        let id = r.id;
        match &e.event {
            spacetimedb_sdk::Event::Reducer(e) => {
                if matches!(e.reducer, Reducer::OpenLootbox(..)) {
                    OperationsPlugin::add(move |world| {
                        Trade::open(id, &egui_context(world).unwrap());
                    });
                }
            }
            _ => {}
        }
    });
    db.wallet().on_update(|_, before, after| {
        let delta = after.amount - before.amount;
        let delta_txt = if delta > 0 {
            format!("+{delta}")
        } else {
            delta.to_string()
        };
        Notification::new(
            "Credits "
                .cstr_c(YELLOW)
                .push(delta_txt.cstr_c(VISIBLE_LIGHT))
                .take(),
        )
        .sfx(SoundEffect::Coin)
        .push_op();
    });
    db.quest().on_insert(|_, d| {
        let text = "New Quest\n".cstr().push(d.cstr()).take();
        Notification::new(text).push_op();
    });
    db.quest().on_update(|_, before, after| {
        let before = before.clone();
        let after = after.clone();
        OperationsPlugin::add(move |world| {
            if before.complete && after.complete {
                return;
            }
            if before.counter < after.counter {
                ShopPlugin::maybe_queue_notification(
                    "Quest Progress:\n"
                        .cstr_c(VISIBLE_BRIGHT)
                        .push(after.cstr())
                        .take(),
                    world,
                )
            }
            if !before.complete && after.complete {
                ShopPlugin::maybe_queue_notification(
                    "Quest Complete!\n"
                        .cstr_c(VISIBLE_BRIGHT)
                        .push(after.cstr())
                        .take(),
                    world,
                )
            }
        });
    });

    db.reward().on_insert(|_, row| {
        if row.owner == player_id() && row.force_open {
            let id = row.id;
            OperationsPlugin::add(move |world| {
                RewardsPlugin::open_reward(id, world);
            });
        }
    });

    fn receive_unit(unit: &FusedUnit) {
        "Unit received: "
            .cstr_c(VISIBLE_LIGHT)
            .push(unit.cstr_expanded())
            .to_notification()
            .sfx(SoundEffect::Inventory)
            .push_op();
    }
    db.unit_item().on_update(|_, before, after| {
        if before.owner != after.owner && after.owner == player_id() {
            receive_unit(&after.unit);
        }
    });
    db.unit_item().on_insert(|_, row| {
        if row.owner == player_id() {
            receive_unit(&row.unit);
        }
    });
    db.unit_item().on_delete(|_, row| {
        if row.owner == player_id() {
            "Unit removed: "
                .cstr_c(VISIBLE_LIGHT)
                .push(row.unit.cstr_expanded())
                .to_notification()
                .sfx(SoundEffect::Inventory)
                .push_op();
        }
    });
    fn notify_shard(delta: i32, unit: &str) {
        let delta = delta.cstr_expanded();
        unit.base_unit()
            .cstr()
            .push(" shards ".cstr_c(VISIBLE_LIGHT))
            .push(delta)
            .to_notification()
            .sfx(SoundEffect::Inventory)
            .push_op();
    }
    db.unit_shard_item().on_update(|_, before, after| {
        if after.owner == player_id() && before.count != after.count {
            notify_shard(after.count as i32 - before.count as i32, &after.unit);
        }
    });
    db.unit_shard_item().on_insert(|_, row| {
        if row.owner == player_id() {
            notify_shard(row.count as i32, &row.unit);
        }
    });
    fn notify_lootbox(delta: i32, kind: &LootboxKind) {
        let delta = delta.cstr_expanded();
        kind.cstr()
            .push(" ".cstr())
            .push(delta)
            .to_notification()
            .sfx(SoundEffect::Inventory)
            .push_op();
    }
    db.lootbox_item().on_update(|_, before, after| {
        if after.owner == player_id() && before.count != after.count {
            notify_lootbox(after.count as i32 - before.count as i32, &after.kind);
        }
    });
    db.lootbox_item().on_insert(|_, row| {
        if row.owner == player_id() {
            notify_lootbox(row.count as i32, &row.kind);
        }
    });
    db.rainbow_shard_item().on_update(|_, before, after| {
        if after.owner == player_id() && before.count != after.count {
            "Rainbow Shards "
                .cstr_rainbow()
                .push((after.count as i32 - before.count as i32).cstr_expanded())
                .to_notification()
                .sfx(SoundEffect::Inventory)
                .push_op();
        }
    });
}
