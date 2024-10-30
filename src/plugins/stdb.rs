use serde_json::to_string_pretty;
use spacetimedb_lib::ser::serde::SerializeWrapper;
use spacetimedb_sdk::{once_on_subscription_applied, subscribe_owned};

use super::*;

#[derive(EnumIter, EnumString, AsRefStr, Hash, PartialEq, Eq, Display, Copy, Clone, Debug)]
pub enum StdbTable {
    GlobalSettings,
    GlobalData,

    TBaseUnit,
    THouse,
    TAbility,
    TStatus,
    TRepresentation,

    TMetaShop,

    TTrade,

    TUser,
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
        [StdbTable::TUser.full(), StdbTable::GlobalData.full()].into()
    }
    pub fn queries_game() -> Vec<StdbQuery> {
        StdbTable::iter().map(|t| t.owner()).collect_vec()
    }
    fn query(self) -> String {
        let table = self.table.as_ref();
        let mut q = format!("select * from {table} ");
        let uid = user_id();
        match self.condition {
            StdbCondition::Full => {}
            StdbCondition::Owner => q.push_str(&format!("where owner = {uid}")),
            StdbCondition::OwnerOrZero => q.push_str(&format!("where owner = {uid} or owner = 0")),
            StdbCondition::Owners(l) => {
                let conditions = l.into_iter().map(|o| format!("owner = {o}")).join(" or ");
                q.push_str(&format!("where {conditions}"));
            }
            StdbCondition::OwnerMacro(m) => {
                let m = m.replace("{uid}", &user_id().to_string());
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

        let queries = subs
            .iter()
            .map(|(t, c)| {
                StdbQuery {
                    table: *t,
                    condition: c.clone(),
                }
                .query()
            })
            .collect_vec();
        info!("Update subscriptions:\n{}", queries.iter().join("\n"));
        subscribe_owned(queries).expect("Failed to subscribe");
        once_on_subscription_applied(move || {
            let on_subscribe = on_subscribe.clone();
            OperationsPlugin::add(on_subscribe);
        });
    }
}

impl StdbTable {
    pub fn get_json_data(self) -> String {
        match self {
            StdbTable::GlobalSettings => {
                to_string_pretty(&SerializeWrapper::new(GlobalSettings::iter().collect_vec()))
            }
            StdbTable::GlobalData => {
                to_string_pretty(&SerializeWrapper::new(GlobalData::iter().collect_vec()))
            }
            StdbTable::TBaseUnit => {
                to_string_pretty(&SerializeWrapper::new(TBaseUnit::iter().collect_vec()))
            }
            StdbTable::THouse => {
                to_string_pretty(&SerializeWrapper::new(THouse::iter().collect_vec()))
            }
            StdbTable::TAbility => {
                to_string_pretty(&SerializeWrapper::new(TAbility::iter().collect_vec()))
            }
            StdbTable::TStatus => {
                to_string_pretty(&SerializeWrapper::new(TStatus::iter().collect_vec()))
            }
            StdbTable::TRepresentation => to_string_pretty(&SerializeWrapper::new(
                TRepresentation::iter().collect_vec(),
            )),
            StdbTable::TMetaShop => {
                to_string_pretty(&SerializeWrapper::new(TMetaShop::iter().collect_vec()))
            }
            StdbTable::TTrade => {
                to_string_pretty(&SerializeWrapper::new(TTrade::iter().collect_vec()))
            }
            StdbTable::TUser => {
                to_string_pretty(&SerializeWrapper::new(TUser::iter().collect_vec()))
            }
            StdbTable::TQuest => {
                to_string_pretty(&SerializeWrapper::new(TQuest::iter().collect_vec()))
            }
            StdbTable::TArenaRun => {
                to_string_pretty(&SerializeWrapper::new(TArenaRun::iter().collect_vec()))
            }
            StdbTable::TArenaRunArchive => to_string_pretty(&SerializeWrapper::new(
                TArenaRunArchive::iter().collect_vec(),
            )),
            StdbTable::TArenaLeaderboard => to_string_pretty(&SerializeWrapper::new(
                TArenaLeaderboard::iter().collect_vec(),
            )),
            StdbTable::TTeam => {
                to_string_pretty(&SerializeWrapper::new(TTeam::iter().collect_vec()))
            }
            StdbTable::TBattle => {
                to_string_pretty(&SerializeWrapper::new(TBattle::iter().collect_vec()))
            }
            StdbTable::TAuction => {
                to_string_pretty(&SerializeWrapper::new(TAuction::iter().collect_vec()))
            }
            StdbTable::TUnitItem => {
                to_string_pretty(&SerializeWrapper::new(TUnitItem::iter().collect_vec()))
            }
            StdbTable::TUnitShardItem => {
                to_string_pretty(&SerializeWrapper::new(TUnitShardItem::iter().collect_vec()))
            }
            StdbTable::TRainbowShardItem => to_string_pretty(&SerializeWrapper::new(
                TRainbowShardItem::iter().collect_vec(),
            )),
            StdbTable::TLootboxItem => {
                to_string_pretty(&SerializeWrapper::new(TLootboxItem::iter().collect_vec()))
            }
            StdbTable::TWallet => {
                to_string_pretty(&SerializeWrapper::new(TWallet::iter().collect_vec()))
            }
            StdbTable::TDailyState => {
                to_string_pretty(&SerializeWrapper::new(TDailyState::iter().collect_vec()))
            }
            StdbTable::TUnitBalance => {
                to_string_pretty(&SerializeWrapper::new(TUnitBalance::iter().collect_vec()))
            }
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
            | StdbTable::TRepresentation
            | StdbTable::TArenaLeaderboard
            | StdbTable::TTeam
            | StdbTable::TUser
            | StdbTable::TArenaRunArchive
            | StdbTable::TMetaShop => self.full(),

            StdbTable::TTrade => StdbQuery {
                table: self,
                condition: StdbCondition::OwnerMacro("a_user = {uid} or b_user = {uid}".into()),
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
            | StdbTable::TBattle
            | StdbTable::TAuction
            | StdbTable::TWallet
            | StdbTable::TDailyState
            | StdbTable::TUnitBalance => StdbQuery {
                table: self,
                condition: StdbCondition::Owner,
            },
        }
    }
}
