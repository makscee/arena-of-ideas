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
    player,
    quest,
    arena_leaderboard,
    wallet,
    daily_state,
    player_tag,
}

#[derive(Clone, PartialEq, Eq, Debug)]
enum StdbCondition {
    Full,
    Owner,
    OwnerOrZero,
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
        self.table.cstr() + &format!("({})", self.condition.cstr())
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
    pub fn get_json_data(self) -> String {
        match self {
            StdbTable::global_settings => to_string_pretty(&SerializeWrapper::new(
                cn().db.global_settings().iter().collect_vec(),
            )),
            StdbTable::global_data => to_string_pretty(&SerializeWrapper::new(
                cn().db.global_data().iter().collect_vec(),
            )),
            StdbTable::player => to_string_pretty(&SerializeWrapper::new(
                cn().db.player().iter().collect_vec(),
            )),
            StdbTable::quest => {
                to_string_pretty(&SerializeWrapper::new(cn().db.quest().iter().collect_vec()))
            }
            StdbTable::arena_leaderboard => to_string_pretty(&SerializeWrapper::new(
                cn().db.arena_leaderboard().iter().collect_vec(),
            )),
            StdbTable::wallet => to_string_pretty(&SerializeWrapper::new(
                cn().db.wallet().iter().collect_vec(),
            )),
            StdbTable::daily_state => to_string_pretty(&SerializeWrapper::new(
                cn().db.daily_state().iter().collect_vec(),
            )),
            StdbTable::player_tag => to_string_pretty(&SerializeWrapper::new(
                cn().db.player_tag().iter().collect_vec(),
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
            | StdbTable::arena_leaderboard
            | StdbTable::player
            | StdbTable::player_tag => Some(self.full()),

            StdbTable::quest => Some(StdbQuery {
                table: self,
                condition: StdbCondition::OwnerOrZero,
            }),

            StdbTable::wallet | StdbTable::daily_state => Some(StdbQuery {
                table: self,
                condition: StdbCondition::Owner,
            }),
        }
    }
}

pub fn reducers_subscriptions(dbc: &DbConnection) {
    let r = dbc.reducers();

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
}

pub fn db_subscriptions() {
    let db = &cn().db;
    db.wallet().on_update(|_, before, after| {
        let delta = after.amount - before.amount;
        let delta_txt = if delta > 0 {
            format!("+{delta}")
        } else {
            delta.to_string()
        };
        Notification::new("Credits ".cstr_c(YELLOW) + &delta_txt.cstr_c(VISIBLE_LIGHT))
            .sfx(SoundEffect::Coin)
            .push_op();
    });
    db.quest().on_insert(|_, d| {
        let text = "New Quest\n".cstr() + &d.cstr();
        Notification::new(text).push_op();
    });
    db.quest().on_update(|_, before, after| {
        let before = before.clone();
        let after = after.clone();
        OperationsPlugin::add(move |world| {
            if before.complete && after.complete {
                return;
            }
            // if before.counter < after.counter {
            //     ShopPlugin::maybe_queue_notification(
            //         "Quest Progress:\n".cstr_c(VISIBLE_BRIGHT) + &after.cstr(),
            //         world,
            //     )
            // }
            // if !before.complete && after.complete {
            //     ShopPlugin::maybe_queue_notification(
            //         "Quest Complete!\n".cstr_c(VISIBLE_BRIGHT) + &after.cstr(),
            //         world,
            //     )
            // }
        });
    });
}
