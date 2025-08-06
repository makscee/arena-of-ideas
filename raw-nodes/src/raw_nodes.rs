use super::*;

struct NCore {
    pub houses: OwnedChildren<NHouse>,
}

struct NPlayers {
    pub players: OwnedChildren<NPlayer>,
}

struct NArena {
    pub floor_pools: OwnedChildren<NFloorPool>,
    pub floor_bosses: OwnedChildren<NFloorBoss>,
}

struct NFloorPool {
    pub floor: i32,
    pub teams: OwnedChildren<NTeam>,
}

struct NFloorBoss {
    pub floor: i32,
    pub team: OwnedParent<NTeam>,
}

struct NPlayer {
    pub player_name: String,
    pub player_data: OwnedParent<NPlayerData>,
    pub identity: OwnedParent<NPlayerIdentity>,
    pub active_match: OwnedParent<NMatch>,
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
    pub color: OwnedParent<NHouseColor>,
    pub action: OwnedParent<NActionAbility>,
    pub status: OwnedParent<NStatusAbility>,
    pub units: OwnedChildren<NUnit>,
}

struct NHouseColor {
    pub color: HexColor,
}

struct NActionAbility {
    pub ability_name: String,
    pub description: OwnedParent<NActionDescription>,
}

struct NActionDescription {
    pub description: String,
    pub effect: OwnedParent<NActionEffect>,
}

struct NActionEffect {
    pub actions: Vec<Action>,
}

struct NStatusAbility {
    pub status_name: String,
    pub description: OwnedParent<NStatusDescription>,
    pub representation: OwnedParent<NStatusRepresentation>,
}

struct NStatusDescription {
    pub description: String,
    pub behavior: OwnedParent<NStatusBehavior>,
}

struct NStatusBehavior {
    pub reactions: Vec<Reaction>,
}

struct NStatusRepresentation {
    pub material: Material,
}

struct NTeam {
    pub houses: OwnedChildren<NHouse>,
    pub fusions: OwnedChildren<NFusion>,
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
    pub shop_offers: Vec<ShopOffer>,
    pub team: OwnedChild<NTeam>,
    pub bench: OwnedChildren<NUnit>,
    pub battles: OwnedChildren<NBattle>,
}

struct NFusion {
    pub units: OwnedParents<NFusionUnit>,
    pub trigger: UnitTriggerRef,
    pub slot: i32,
    pub pwr: i32,
    pub hp: i32,
    pub dmg: i32,
    pub units_limit: i32,
    pub actions_limit: i32,
}

struct NFusionUnit {
    pub index: i32,
    pub actions: UnitActionRange,
    pub unit: OwnedParent<NUnit>,
}

struct NUnit {
    pub unit_name: String,
    pub description: OwnedParent<NUnitDescription>,
    pub stats: OwnedParent<NUnitStats>,
    pub state: OwnedParent<NUnitState>,
    pub house: LinkedParent<NHouse>,
}

struct NUnitDescription {
    pub description: String,
    pub representation: OwnedParent<NUnitRepresentation>,
    pub behavior: OwnedParent<NUnitBehavior>,
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
