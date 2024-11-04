use std::sync::RwLockWriteGuard;

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
pub trait TableSingletonExt: TableType {
    fn current() -> Self {
        *Self::get_current().unwrap()
    }
    fn get_current() -> Option<Box<Self>> {
        Self::iter().exactly_one().ok().map(|d| Box::new(d))
    }
}

impl TableSingletonExt for GlobalData {}
impl TableSingletonExt for GlobalSettings {}
impl TableSingletonExt for TArenaRun {}
impl TableSingletonExt for TWallet {}
impl TableSingletonExt for TDailyState {
    fn current() -> Self {
        *Self::get_current().unwrap_or_else(|| {
            Box::new(Self {
                owner: player_id(),
                ranked_cost: 0,
                const_cost: 0,
                quests_taken: default(),
            })
        })
    }

    fn get_current() -> Option<Box<Self>> {
        Self::iter().exactly_one().ok().map(|d| Box::new(d))
    }
}

pub trait StdbStatusExt {
    fn on_success(&self, f: impl FnOnce(&mut World) + Send + Sync + 'static);
    fn notify_error(&self);
}

impl StdbStatusExt for spacetimedb_sdk::reducer::Status {
    fn on_success(&self, f: impl FnOnce(&mut World) + Send + Sync + 'static) {
        match self {
            StdbStatus::Committed => OperationsPlugin::add(f),
            StdbStatus::Failed(e) => e.notify_error_op(),
            _ => panic!(),
        }
    }
    fn notify_error(&self) {
        match self {
            StdbStatus::Committed => {}
            StdbStatus::Failed(e) => e.notify_error_op(),
            _ => panic!(),
        }
    }
}

pub trait GIDExt {
    fn get_team(self) -> TTeam;
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
        TTeam::find_by_id(self)
            .with_context(|| format!("Failed to find Team#{self}"))
            .unwrap()
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
            return TPlayer {
                id: 0,
                name: "...".into(),
                identities: default(),
                pass_hash: default(),
                online: default(),
                last_login: default(),
            };
        }
        TPlayer::find_by_id(self)
            .with_context(|| format!("Failed to find Player#{self}"))
            .unwrap()
    }
    fn unit_item(self) -> TUnitItem {
        TUnitItem::find_by_id(self)
            .with_context(|| format!("Failed to find UnitItem#{self}"))
            .unwrap()
    }
    fn unit_shard_item(self) -> TUnitShardItem {
        TUnitShardItem::find_by_id(self)
            .with_context(|| format!("Failed to find UnitShardItem#{self}"))
            .unwrap()
    }
    fn rainbow_shard_item(self) -> TRainbowShardItem {
        TRainbowShardItem::find_by_id(self)
            .with_context(|| format!("Failed to find RainbowShardItem#{self}"))
            .unwrap()
    }
    fn lootbox_item(self) -> TLootboxItem {
        TLootboxItem::find_by_id(self)
            .with_context(|| format!("Failed to find LootboxItem#{self}"))
            .unwrap()
    }
}

pub trait BaseUnitExt {
    fn base(&self) -> &str;
    fn base_unit(&self) -> TBaseUnit {
        TBaseUnit::find_by_name(self.base().into()).unwrap()
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
impl GameMode {
    pub fn weak_eq(&self, other: &Self) -> bool {
        mem::discriminant(self) == mem::discriminant(other)
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
        }
    }
}
impl Default for UnitPool {
    fn default() -> Self {
        Self::Game
    }
}
