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
    pub units: RefMultiple<NUnit>,
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
    pub effect: Component<NAbilityEffect>,
}

#[derive(Node)]
#[content]
pub struct NAbilityEffect {
    pub effect: Effect,
}

#[derive(Node)]
#[content]
#[named(status_name)]
pub struct NStatusMagic {
    #[var]
    pub status_name: String,
    pub behavior: Component<NStatusBehavior>,
    pub state: Component<NState>,
}

#[derive(Node)]
#[content]
pub struct NStatusBehavior {
    pub reactions: Vec<Reaction>,
    pub representation: Component<NStatusRepresentation>,
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
pub struct NUnitState {
    #[var]
    pub stax: i32,
    #[var]
    pub dmg: i32,
}

#[derive(Node)]
pub struct NTeam {
    pub slots: OwnedMultiple<NTeamSlot>,
    pub houses: OwnedMultiple<NHouse>,
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
    pub battle_history: Vec<u64>,
    pub pending_battle: Option<u64>,
    pub fusion: Option<(u64, u64, Vec<PackedNodes>)>,

    pub shop_pool: Owned<NShopPool>,
    pub slots: OwnedMultiple<NTeamSlot>,
    pub bench: OwnedMultiple<NUnit>,
}

#[derive(Node)]
pub struct NShopPool {
    pub houses: OwnedMultiple<NHouse>,
    pub units: OwnedMultiple<NUnit>,
}

#[derive(Node)]
pub struct NTeamSlot {
    #[var]
    pub index: i32,
    pub unit: Owned<NUnit>,
}

#[derive(Node)]
#[content]
#[named(unit_name)]
pub struct NUnit {
    #[var]
    pub unit_name: String,
    pub behavior: Component<NUnitBehavior>,
    pub state: Component<NUnitState>,
}

#[derive(Node)]
#[content]
pub struct NUnitBehavior {
    pub reactions: Vec<Reaction>,
    pub stats: Component<NUnitStats>,
    pub representation: Component<NUnitRepresentation>,
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
pub struct NUnitRepresentation {
    pub material: Material,
}

#[derive(Node)]
pub struct NRepresentation {
    pub material: Material,
}
