use crate::*;
pub use proc_macros::Node;

#[derive(Node)]
pub struct NArena {
    pub last_floor: i32,
    pub floor_pools: OwnedMultiple<NFloorPool>,
    pub floor_bosses: OwnedMultiple<NFloorBoss>,
}

#[derive(Node)]
pub struct NFloorPool {
    #[var]
    pub floor: i32,
    pub teams: OwnedMultiple<NTeam>,
}

#[derive(Node)]
pub struct NFloorBoss {
    #[var]
    pub floor: i32,
    pub team: Owned<NTeam>,
}

#[derive(Node)]
pub struct NPlayer {
    #[var]
    pub player_name: String,
    pub player_data: Component<NPlayerData>,
    pub identity: Component<NPlayerIdentity>,
    pub active_match: Component<NMatch>,
}

#[derive(Node)]
pub struct NPlayerData {
    pub pass_hash: Option<String>,
    #[var]
    pub online: bool,
    pub last_login: u64,
}

#[derive(Node)]
pub struct NPlayerIdentity {
    pub data: Option<String>,
}

#[derive(Node)]
#[content]
#[named(house_name)]
pub struct NHouse {
    #[var]
    pub house_name: String,
    pub color: Component<NHouseColor>,
    pub ability: Component<NAbilityMagic>,
    pub status: Component<NStatusMagic>,
    pub state: Component<NState>,
    pub units: OwnedMultiple<NUnit>,
}

#[derive(Node)]
#[content]
pub struct NHouseColor {
    #[var]
    pub color: HexColor,
}

#[derive(Node)]
#[content]
#[named(ability_name)]
pub struct NAbilityMagic {
    #[var]
    pub ability_name: String,
    pub description: Component<NAbilityDescription>,
}

#[derive(Node)]
#[content]
pub struct NAbilityDescription {
    #[var]
    pub description: String,
    pub effect: Component<NAbilityEffect>,
}

#[derive(Node)]
#[content]
pub struct NAbilityEffect {
    pub actions: Vec<Action>,
}

#[derive(Node)]
#[content]
#[named(status_name)]
pub struct NStatusMagic {
    #[var]
    pub status_name: String,
    pub description: Component<NStatusDescription>,
    pub representation: Component<NStatusRepresentation>,
    pub state: Component<NState>,
}

#[derive(Node)]
#[content]
pub struct NStatusDescription {
    #[var]
    pub description: String,
    pub behavior: Component<NStatusBehavior>,
}

#[derive(Node)]
#[content]
pub struct NStatusBehavior {
    pub reactions: Vec<Reaction>,
}

#[derive(Node)]
#[content]
pub struct NStatusRepresentation {
    pub material: Material,
}

#[derive(Node)]
pub struct NState {
    #[var]
    pub stax: i32,
}

#[derive(Node)]
pub struct NTeam {
    pub houses: OwnedMultiple<NHouse>,
    pub fusions: OwnedMultiple<NFusion>,
}

#[derive(Node)]
pub struct NBattle {
    pub team_left: u64,
    pub team_right: u64,
    pub ts: u64,
    pub hash: u64,
    pub result: Option<bool>,
}

#[derive(Node)]
pub struct NMatch {
    #[var]
    pub g: i32,
    #[var]
    pub floor: i32,
    #[var]
    pub lives: i32,
    #[var]
    pub active: bool,
    pub state: MatchState,
    pub shop_offers: Vec<ShopOffer>,
    pub team: Owned<NTeam>,
    pub battles: OwnedMultiple<NBattle>,
}

#[derive(Node)]
pub struct NFusion {
    pub slots: OwnedMultiple<NFusionSlot>,
    pub trigger_unit: Ref<NUnit>,
    #[var]
    pub index: i32,
    #[var]
    pub pwr: i32,
    #[var]
    pub hp: i32,
    #[var]
    pub dmg: i32,
    pub actions_limit: i32,
}

#[derive(Node)]
pub struct NFusionSlot {
    #[var]
    pub index: i32,
    pub actions: UnitActionRange,
    pub unit: Ref<NUnit>,
}

#[derive(Node)]
#[content]
#[named(unit_name)]
pub struct NUnit {
    #[var]
    pub unit_name: String,
    pub description: Component<NUnitDescription>,
    pub stats: Component<NUnitStats>,
    pub state: Component<NState>,
}

#[derive(Node)]
#[content]
pub struct NUnitDescription {
    #[var]
    pub description: String,
    pub magic_type: MagicType,
    pub trigger: Trigger,
    pub representation: Component<NUnitRepresentation>,
    pub behavior: Component<NUnitBehavior>,
}

#[derive(Node)]
#[content]
pub struct NUnitStats {
    #[var]
    pub pwr: i32,
    #[var]
    pub hp: i32,
}

#[derive(Node)]
#[content]
pub struct NUnitBehavior {
    pub reaction: Reaction,
    pub magic_type: MagicType,
}

#[derive(Node)]
#[content]
pub struct NUnitRepresentation {
    pub material: Material,
}
