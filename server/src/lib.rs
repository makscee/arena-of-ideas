mod auth;
mod content;
mod match_reducer;
mod seed;
mod voting;

use spacetimedb::ReducerContext;

#[spacetimedb::reducer(init)]
pub fn init(ctx: &ReducerContext) {
    seed::seed_primordial_abilities(ctx);
    log::info!("Arena of Ideas server initialized");
}

#[spacetimedb::reducer(client_connected)]
pub fn client_connected(_ctx: &ReducerContext) {}

#[spacetimedb::reducer(client_disconnected)]
pub fn client_disconnected(_ctx: &ReducerContext) {}

// ===== Custom Types =====

#[derive(spacetimedb::SpacetimeType, Clone, Debug, PartialEq)]
pub enum ContentStatus {
    Draft,
    Incubator,
    Active,
    Retired,
}

#[derive(spacetimedb::SpacetimeType, Clone, Debug, PartialEq)]
pub enum Trigger {
    BattleStart,
    TurnEnd,
    BeforeDeath,
    AllyDeath,
    BeforeStrike,
    AfterStrike,
    DamageTaken,
    DamageDealt,
}

#[derive(spacetimedb::SpacetimeType, Clone, Debug, PartialEq)]
pub enum TargetType {
    RandomEnemy,
    AllEnemies,
    RandomAlly,
    AllAllies,
    Owner,
    All,
    Attacker,
    AdjacentBack,
    AdjacentFront,
}

#[derive(spacetimedb::SpacetimeType, Clone, Debug, PartialEq)]
pub enum GenStatus {
    Pending,
    Processing,
    Done,
    Failed,
}

#[derive(spacetimedb::SpacetimeType, Clone, Debug, PartialEq)]
pub enum GenTargetKind {
    Ability,
    Unit,
}

// ===== Tables =====

#[spacetimedb::table(accessor = player, public)]
pub struct Player {
    #[primary_key]
    pub identity: spacetimedb::Identity,
    pub name: String,
    pub created_at: spacetimedb::Timestamp,
}

#[spacetimedb::table(accessor = ability, public)]
pub struct Ability {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    #[unique]
    pub name: String,
    pub description: String,
    pub target_type: TargetType,
    pub effect_script: String,
    pub parent_a: u64,
    pub parent_b: u64,
    pub rating: i32,
    pub status: ContentStatus,
    pub season: u32,
    pub created_by: spacetimedb::Identity,
    pub created_at: spacetimedb::Timestamp,
}

#[spacetimedb::table(accessor = unit, public)]
pub struct Unit {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    #[unique]
    pub name: String,
    pub description: String,
    pub hp: i32,
    pub pwr: i32,
    pub tier: u8,
    pub trigger: Trigger,
    pub abilities: Vec<u64>,
    pub painter_script: String,
    pub rating: i32,
    pub status: ContentStatus,
    pub created_by: spacetimedb::Identity,
    pub created_at: spacetimedb::Timestamp,
}

#[spacetimedb::table(accessor = vote, public)]
pub struct Vote {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub player: spacetimedb::Identity,
    /// "ability" or "unit"
    pub entity_kind: String,
    pub entity_id: u64,
    /// +1 or -1
    pub value: i8,
    pub created_at: spacetimedb::Timestamp,
}

#[spacetimedb::table(accessor = gen_request, public)]
pub struct GenRequest {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub player: spacetimedb::Identity,
    pub target_kind: GenTargetKind,
    pub prompt: String,
    pub parent_a: u64,
    pub parent_b: u64,
    pub context_id: u64,
    pub status: GenStatus,
    pub created_at: spacetimedb::Timestamp,
}

#[spacetimedb::table(accessor = gen_result, public)]
pub struct GenResult {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub request_id: u64,
    pub data: String,
    pub explanation: String,
    pub created_at: spacetimedb::Timestamp,
}

#[spacetimedb::table(accessor = season, public)]
pub struct Season {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub active_ability_ids: Vec<u64>,
    pub created_at: spacetimedb::Timestamp,
}

#[spacetimedb::table(accessor = feature_request, public)]
pub struct FeatureRequest {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub player: spacetimedb::Identity,
    pub description: String,
    pub rating: i32,
    pub status: String,
    pub created_at: spacetimedb::Timestamp,
}

// ===== Match Types =====

#[derive(spacetimedb::SpacetimeType, Clone, Debug)]
pub struct TeamSlot {
    pub unit_id: u64,
    pub copies: u8,
    pub bonus_hp: i32,
    pub bonus_pwr: i32,
    pub is_fused: bool,
    pub fused_trigger: String,
    pub fused_abilities: Vec<u64>,
    pub fused_tier: u8,
}

#[spacetimedb::table(accessor = game_match, public)]
pub struct GameMatch {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub player: spacetimedb::Identity,
    pub floor: u8,
    pub gold: i32,
    pub lives: i32,
    pub team: Vec<TeamSlot>,
    pub shop_offers: Vec<u64>,
    pub created_at: spacetimedb::Timestamp,
}
