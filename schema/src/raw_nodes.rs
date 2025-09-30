use crate::*;
pub use proc_macros::node;

#[node]
pub struct NArena {
    pub floor_pools: Owned<Vec<NFloorPool>>,
    pub floor_bosses: Owned<Vec<NFloorBoss>>,
}

#[node]
pub struct NFloorPool {
    pub floor: i32,
    pub teams: Owned<Vec<NTeam>>,
}

#[node]
pub struct NFloorBoss {
    pub floor: i32,
    pub team: Component<NTeam>,
}

#[node]
pub struct NPlayer {
    pub player_name: String,
    pub player_data: Component<NPlayerData>,
    pub identity: Component<NPlayerIdentity>,
    pub active_match: Component<NMatch>,
}

#[node]
pub struct NPlayerData {
    pub pass_hash: Option<String>,
    pub online: bool,
    pub last_login: u64,
}

#[node]
pub struct NPlayerIdentity {
    pub data: Option<String>,
}

#[node(content, name)]
pub struct NHouse {
    pub house_name: String,
    pub color: Component<NHouseColor>,
    pub ability: Component<NAbilityMagic>,
    pub status: Component<NStatusMagic>,
    pub units: Owned<Vec<NUnit>>,
}

#[node(content)]
pub struct NHouseColor {
    pub color: HexColor,
}

#[node(content, name)]
pub struct NAbilityMagic {
    pub ability_name: String,
    pub description: Component<NAbilityDescription>,
}

#[node(content)]
pub struct NAbilityDescription {
    pub description: String,
    pub effect: Component<NAbilityEffect>,
}

#[node(content)]
pub struct NAbilityEffect {
    pub actions: Vec<Action>,
}

#[node(content, name)]
pub struct NStatusMagic {
    pub status_name: String,
    pub description: Component<NStatusDescription>,
    pub representation: Component<NStatusRepresentation>,
}

#[node(content)]
pub struct NStatusDescription {
    pub description: String,
    pub behavior: Component<NStatusBehavior>,
}

#[node(content)]
pub struct NStatusBehavior {
    pub reactions: Vec<Reaction>,
}

#[node(content)]
pub struct NStatusRepresentation {
    pub material: Material,
}

#[node]
pub struct NTeam {
    pub houses: Owned<Vec<NHouse>>,
    pub fusions: Owned<Vec<NFusion>>,
}

#[node]
pub struct NBattle {
    pub team_left: u64,
    pub team_right: u64,
    pub ts: u64,
    pub hash: u64,
    pub result: Option<bool>,
}

#[node]
pub struct NMatch {
    pub g: i32,
    pub floor: i32,
    pub lives: i32,
    pub active: bool,
    pub shop_offers: Vec<ShopOffer>,
    pub team: Component<NTeam>,
    pub battles: Owned<Vec<NBattle>>,
}

#[node]
pub struct NFusion {
    pub slots: Owned<Vec<NFusionSlot>>,
    pub trigger_unit: u64,
    pub index: i32,
    pub pwr: i32,
    pub hp: i32,
    pub dmg: i32,
    pub actions_limit: i32,
}

#[node]
pub struct NFusionSlot {
    pub index: i32,
    pub actions: UnitActionRange,
    pub unit: Ref<NUnit>,
}

#[node(content, name)]
pub struct NUnit {
    pub unit_name: String,
    pub description: Component<NUnitDescription>,
    pub stats: Component<NUnitStats>,
    pub state: Component<NUnitState>,
}

#[node(content)]
pub struct NUnitDescription {
    pub description: String,
    pub magic_type: MagicType,
    pub trigger: Trigger,
    pub representation: Component<NUnitRepresentation>,
    pub behavior: Component<NUnitBehavior>,
}

#[node(content)]
pub struct NUnitStats {
    pub pwr: i32,
    pub hp: i32,
}

#[node]
pub struct NUnitState {
    pub stacks: i32,
}

#[node(content)]
pub struct NUnitBehavior {
    pub reaction: Reaction,
    pub magic_type: MagicType,
}

#[node(content)]
pub struct NUnitRepresentation {
    pub material: Material,
}
