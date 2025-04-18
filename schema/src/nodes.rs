use super::*;

struct NCore {
    pub houses: NodeChildren<NHouse>,
}

struct NPlayers {
    pub players: NodeChildren<NPlayer>,
}

struct NIncubator {
    pub houses: NodeChildren<NHouse>,
    pub house_colors: NodeChildren<NHouseColor>,
    pub units: NodeChildren<NUnit>,
    pub unit_descriptions: NodeChildren<NUnitDescription>,
    pub unit_stats: NodeChildren<NUnitStats>,
    pub abilities: NodeChildren<NAbilityMagic>,
    pub ability_descriptions: NodeChildren<NAbilityDescription>,
    pub ability_effects: NodeChildren<NAbilityEffect>,
    pub statuses: NodeChildren<NStatusMagic>,
    pub status_descriptions: NodeChildren<NStatusDescription>,
    pub representations: NodeChildren<NRepresentation>,
    pub reactions: NodeChildren<NBehavior>,
}

struct NArena {
    pub floor_pools: NodeChildren<NFloorPool>,
    pub floor_bosses: NodeChildren<NFloorBoss>,
}

struct NFloorPool {
    pub floor: i32,
    pub teams: NodeChildren<NTeam>,
}

struct NFloorBoss {
    pub floor: i32,
    pub team: NodeComponent<NTeam>,
}

struct NPlayer {
    pub player_name: String,
    pub player_data: NodeComponent<NPlayerData>,
    pub identity: NodeComponent<NPlayerIdentity>,
    pub active_match: NodeComponent<NMatch>,
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
    pub color: NodeComponent<NHouseColor>,
    pub ability_magic: NodeComponent<NAbilityMagic>,
    pub status_magic: NodeComponent<NStatusMagic>,
    pub units: NodeChildren<NUnit>,
}

struct NHouseColor {
    pub color: HexColor,
}

struct NAbilityMagic {
    pub ability_name: String,
    pub description: NodeComponent<NAbilityDescription>,
}

struct NAbilityDescription {
    pub description: String,
    pub effect: NodeComponent<NAbilityEffect>,
}

struct NAbilityEffect {
    pub actions: Actions,
}

struct NStatusMagic {
    pub status_name: String,
    pub description: NodeComponent<NStatusDescription>,
    pub representation: NodeComponent<NRepresentation>,
}

struct NStatusDescription {
    pub description: String,
    pub behavior: NodeComponent<NBehavior>,
}

struct NTeam {
    pub owner: u64,
    pub houses: NodeChildren<NHouse>,
    pub fusions: NodeChildren<NFusion>,
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
    pub shop_case: NodeChildren<NShopCaseUnit>,
    pub team: NodeComponent<NTeam>,
    pub battles: NodeChildren<NBattle>,
}

struct NShopCaseUnit {
    pub unit: u64,
    pub price: i32,
    pub sold: bool,
}

struct NFusion {
    pub units: Vec<u64>,
    pub behavior: Vec<(UnitTriggerRef, Vec<UnitActionRef>)>,
    pub slot: i32,
    pub stats: NodeComponent<NFusionStats>,
}

struct NUnit {
    pub unit_name: String,
    pub description: NodeComponent<NUnitDescription>,
    pub stats: NodeComponent<NUnitStats>,
}

struct NUnitDescription {
    pub description: String,
    pub representation: NodeComponent<NRepresentation>,
    pub behavior: NodeComponent<NBehavior>,
}

struct NUnitStats {
    pub pwr: i32,
    pub hp: i32,
}

struct NFusionStats {
    pub pwr: i32,
    pub hp: i32,
    pub dmg: i32,
}

struct NBehavior {
    pub reactions: Vec<Reaction>,
}

struct NRepresentation {
    pub material: Material,
}
