use super::*;

struct NCore {
    pub houses: ChildComponents<NHouse>,
}

struct NPlayers {
    pub players: ChildComponents<NPlayer>,
}

struct NArena {
    pub floor_pools: ChildComponents<NFloorPool>,
    pub floor_bosses: ChildComponents<NFloorBoss>,
}

struct NFloorPool {
    pub floor: i32,
    pub teams: ChildComponents<NTeam>,
}

struct NFloorBoss {
    pub floor: i32,
    pub team: ParentComponent<NTeam>,
}

struct NPlayer {
    pub player_name: String,
    pub player_data: ParentComponent<NPlayerData>,
    pub identity: ParentComponent<NPlayerIdentity>,
    pub active_match: ParentComponent<NMatch>,
}

struct NPlayerData {
    pub pass_hash: Option<String>,
    pub online: bool,
    pub last_login: u64,
}

struct NPlayerIdentity {
    pub data: Option<String>,
}

struct NHouse {
    pub house_name: String,
    pub color: ParentComponent<NHouseColor>,
    pub ability_magic: ParentComponent<NAbilityMagic>,
    pub status_magic: ParentComponent<NStatusMagic>,
    pub units: ChildComponents<NUnit>,
}

struct NHouseColor {
    pub color: HexColor,
}

struct NAbilityMagic {
    pub ability_name: String,
    pub description: ParentComponent<NAbilityDescription>,
}

struct NAbilityDescription {
    pub description: String,
    pub effect: ParentComponent<NAbilityEffect>,
}

struct NAbilityEffect {
    pub actions: Vec<Action>,
}

struct NStatusMagic {
    pub status_name: String,
    pub description: ParentComponent<NStatusDescription>,
    pub representation: ParentComponent<NStatusRepresentation>,
}

struct NStatusDescription {
    pub description: String,
    pub behavior: ParentComponent<NStatusBehavior>,
}

struct NStatusBehavior {
    pub reactions: Vec<Reaction>,
}

struct NStatusRepresentation {
    pub material: Material,
}

struct NTeam {
    pub houses: ChildComponents<NHouse>,
    pub fusions: ChildComponents<NFusion>,
}

struct NBattle {
    pub team_left: u64,
    pub team_right: u64,
    pub ts: u64,
    pub hash: u64,
    pub result: Option<bool>,
}

struct NMatch {
    pub g: i32,
    pub floor: i32,
    pub round: i32,
    pub lives: i32,
    pub active: bool,
    pub team: ParentComponent<NTeam>,
    pub hand: Vec<(CardKind, u64)>,
    pub shop_offers: Vec<ShopOffer>,
    pub battles: ChildComponents<NBattle>,
}

struct NFusion {
    pub units: ParentLinks<NUnit>,
    pub behavior: Vec<(UnitTriggerRef, Vec<UnitActionRef>)>,
    pub slot: i32,
    pub pwr: i32,
    pub hp: i32,
    pub dmg: i32,
    pub lvl: i32,
    pub action_limit: i32,
}

struct NUnit {
    pub unit_name: String,
    pub description: ParentComponent<NUnitDescription>,
    pub stats: ParentComponent<NUnitStats>,
    pub state: ParentComponent<NUnitState>,
}

struct NUnitDescription {
    pub description: String,
    pub representation: ParentComponent<NUnitRepresentation>,
    pub behavior: ParentComponent<NUnitBehavior>,
}

struct NUnitStats {
    pub pwr: i32,
    pub hp: i32,
}

struct NUnitState {
    pub xp: i32,
    pub lvl: i32,
    pub rarity: i32,
}

struct NUnitBehavior {
    pub reactions: Vec<Reaction>,
}

struct NUnitRepresentation {
    pub material: Material,
}
