use std::sync::RwLockWriteGuard;

use spacetimedb_sdk::{Event, Table};

use super::*;

pub trait EntityExt {
    fn get_parent(self, world: &World) -> Option<Entity>;
    fn get_parent_query(self, query: &Query<&Parent>) -> Option<Entity>;
    fn faction(self, world: &World) -> Faction;
}

impl EntityExt for Entity {
    fn get_parent(self, world: &World) -> Option<Entity> {
        world.get::<Parent>(self).map(|p| p.get())
    }
    fn get_parent_query(self, query: &Query<&Parent>) -> Option<Entity> {
        query.get(self).ok().map(|p| p.get())
    }
    fn faction(self, world: &World) -> Faction {
        Context::new(self)
            .get_value(VarName::Faction, world)
            .unwrap()
            .get_faction()
            .unwrap()
    }
}
pub trait TableSingletonExt: Table {
    fn current(&self) -> Self::Row {
        *Self::get_current(self).unwrap()
    }
    fn get_current(&self) -> Option<Box<Self::Row>> {
        Self::iter(self).exactly_one().ok().map(|d| Box::new(d))
    }
}

impl TableSingletonExt for GlobalDataTableHandle<'static> {}
impl TableSingletonExt for GlobalSettingsTableHandle<'static> {}
impl TableSingletonExt for ArenaRunTableHandle<'static> {}
impl TableSingletonExt for WalletTableHandle<'static> {}
impl TableSingletonExt for DailyStateTableHandle<'static> {
    fn current(&self) -> Self::Row {
        *Self::get_current(self).unwrap_or_else(|| {
            Box::new(Self::Row {
                owner: player_id(),
                ranked_cost: 0,
                const_cost: 0,
                quests_taken: default(),
                meta_shop_discount_spent: false,
            })
        })
    }

    fn get_current(&self) -> Option<Box<Self::Row>> {
        Self::iter(self).exactly_one().ok().map(|d| Box::new(d))
    }
}

pub trait StdbStatusExt {
    fn on_success(&self, f: impl FnOnce(&mut World) + Send + Sync + 'static);
    fn notify_error(&self);
}

impl<R> StdbStatusExt for Event<R> {
    fn on_success(&self, f: impl FnOnce(&mut World) + Send + Sync + 'static) {
        match self {
            Event::Reducer(r) => match &r.status {
                spacetimedb_sdk::Status::Committed => OperationsPlugin::add(f),
                spacetimedb_sdk::Status::Failed(e) => e.notify_error_op(),
                _ => panic!(),
            },
            Event::SubscribeApplied | Event::UnsubscribeApplied => OperationsPlugin::add(f),
            Event::SubscribeError(e) => e.to_string().notify_error_op(),
            Event::UnknownTransaction => "Unknown transaction".notify_error_op(),
            _ => panic!(),
        }
    }
    fn notify_error(&self) {
        match self {
            Event::Reducer(r) => match &r.status {
                spacetimedb_sdk::Status::Committed => {}
                spacetimedb_sdk::Status::Failed(e) => e.notify_error_op(),
                _ => panic!(),
            },
            Event::SubscribeError(e) => e.to_string().notify_error_op(),
            Event::UnknownTransaction => "Unknown transaction".notify_error_op(),
            _ => panic!(),
        }
    }
}

pub trait GIDExt {
    fn get_team(self) -> TTeam;
    fn try_get_team(self) -> Option<TTeam>;
    fn get_team_cached(self) -> TTeam;
    fn get_player(self) -> TPlayer;
    fn unit_item(self) -> TUnitItem;
    fn unit_shard_item(self) -> TUnitShardItem;
    fn rainbow_shard_item(self) -> TRainbowShardItem;
    fn lootbox_item(self) -> TLootboxItem;
}

#[derive(Default)]
struct StdbCache {
    teams: HashMap<u64, TTeam>,
}

static STDB_CACHE: OnceCell<RwLock<StdbCache>> = OnceCell::new();
fn stdb_cache_get_mut() -> RwLockWriteGuard<'static, StdbCache> {
    STDB_CACHE.get_or_init(|| default()).write().unwrap()
}

pub fn stdb_cache_reset() {
    *stdb_cache_get_mut() = default()
}

impl GIDExt for u64 {
    fn get_team(self) -> TTeam {
        if self == 0 {
            return TTeam {
                id: 0,
                owner: 0,
                units: default(),
                name: default(),
                pool: TeamPool::Owned,
            };
        }
        self.try_get_team()
            .with_context(|| format!("Failed to find Team#{self}"))
            .unwrap()
    }
    fn try_get_team(self) -> Option<TTeam> {
        cn().db.team().id().find(&self)
    }
    fn get_team_cached(self) -> TTeam {
        let mut cache = stdb_cache_get_mut();
        if let Some(team) = cache.teams.get(&self) {
            return team.clone();
        } else {
            let team = self.get_team();
            cache.teams.insert(self, team.clone());
            team
        }
    }
    fn get_player(self) -> TPlayer {
        if self == 0 {
            return TPlayer::default();
        }
        cn().db.player().id().find(&self).unwrap_or_default()
    }
    fn unit_item(self) -> TUnitItem {
        cn().db
            .unit_item()
            .id()
            .find(&self)
            .with_context(|| format!("Failed to find UnitItem#{self}"))
            .unwrap()
    }
    fn unit_shard_item(self) -> TUnitShardItem {
        cn().db
            .unit_shard_item()
            .id()
            .find(&self)
            .with_context(|| format!("Failed to find UnitShardItem#{self}"))
            .unwrap()
    }
    fn rainbow_shard_item(self) -> TRainbowShardItem {
        cn().db
            .rainbow_shard_item()
            .id()
            .find(&self)
            .with_context(|| format!("Failed to find RainbowShardItem#{self}"))
            .unwrap()
    }
    fn lootbox_item(self) -> TLootboxItem {
        cn().db
            .lootbox_item()
            .id()
            .find(&self)
            .with_context(|| format!("Failed to find LootboxItem#{self}"))
            .unwrap()
    }
}

pub trait BaseUnitExt {
    fn base(&self) -> &str;
    fn base_unit(&self) -> TBaseUnit {
        cn().db
            .base_unit()
            .name()
            .find(&self.base().into())
            .unwrap()
    }
}

impl BaseUnitExt for FusedUnit {
    fn base(&self) -> &str {
        &self.bases[0]
    }
}
impl BaseUnitExt for String {
    fn base(&self) -> &str {
        self
    }
}

impl TTeam {
    pub fn hover_label(&self, ui: &mut Ui, world: &mut World) {
        let resp = self
            .cstr()
            .as_label(ui)
            .sense(Sense::click())
            .selectable(false)
            .ui(ui);
        if resp.hovered() {
            cursor_window(ui.ctx(), |ui| {
                Frame {
                    inner_margin: Margin::same(8.0),
                    rounding: Rounding::same(13.0),
                    fill: BG_TRANSPARENT,
                    ..default()
                }
                .show(ui, |ui| {
                    self.show(ui, world);
                });
            });
            if resp.clicked() {
                let packed = PackedTeam::from_id(self.id);
                let s = ron::to_string(&packed).unwrap();
                copy_to_clipboard(&s, world);
                Notification::new(
                    format!("Team#{} copied to clipboard", self.id).cstr_c(VISIBLE_LIGHT),
                )
                .push(world);
            }
        }
    }
}

impl Default for GameMode {
    fn default() -> Self {
        Self::ArenaNormal
    }
}

impl ToString for GameMode {
    fn to_string(&self) -> String {
        match self {
            GameMode::ArenaNormal => "Normal".into(),
            GameMode::ArenaRanked => "Ranked".into(),
            GameMode::ArenaConst => "Const".into(),
        }
    }
}
impl Eq for GameMode {}
impl Hash for GameMode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}
impl Copy for GameMode {}

impl From<u64> for GameMode {
    fn from(value: u64) -> Self {
        match value {
            0 => GameMode::ArenaNormal,
            1 => GameMode::ArenaRanked,
            2 => GameMode::ArenaConst,
            _ => panic!(),
        }
    }
}
impl Into<u64> for GameMode {
    fn into(self) -> u64 {
        match self {
            GameMode::ArenaNormal => 0,
            GameMode::ArenaRanked => 1,
            GameMode::ArenaConst => 2,
        }
    }
}

impl Default for GameData {
    fn default() -> Self {
        Self {
            global_data: default(),
            global_settings: default(),
            ability: default(),
            arena_leaderboard: default(),
            arena_run: default(),
            arena_run_archive: default(),
            auction: default(),
            base_unit: default(),
            battle: default(),
            daily_state: default(),
            house: default(),
            lootbox_item: default(),
            meta_shop: default(),
            quest: default(),
            rainbow_shard_item: default(),
            status: default(),
            team: default(),
            trade: default(),
            unit_balance: default(),
            unit_item: default(),
            unit_shard_item: default(),
            player: default(),
            wallet: default(),
            incubator: default(),
            incubator_vote: default(),
            incubator_favorite: default(),
            player_stats: default(),
            player_game_stats: default(),
            global_event: default(),
        }
    }
}
impl Default for UnitPool {
    fn default() -> Self {
        Self::Game
    }
}
impl Default for TPlayer {
    fn default() -> Self {
        Self {
            id: 0,
            name: "...".into(),
            identities: default(),
            pass_hash: default(),
            online: default(),
            last_login: default(),
        }
    }
}
impl EventContext {
    pub fn check_identity(&self) -> bool {
        match &self.event {
            Event::Reducer(r) => r.caller_identity == player_identity(),
            _ => true,
        }
    }
}
