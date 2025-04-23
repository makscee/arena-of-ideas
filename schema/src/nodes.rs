use super::*;

struct NCore {
    pub houses: LinkMany<NHouse>,
}

struct NPlayers {
    pub players: LinkMany<NPlayer>,
}

struct NArena {
    pub floor_pools: LinkMany<NFloorPool>,
    pub floor_bosses: LinkMany<NFloorBoss>,
}

struct NFloorPool {
    pub floor: i32,
    pub teams: LinkMany<NTeam>,
}

struct NFloorBoss {
    pub floor: i32,
    pub team: LinkOne<NTeam>,
}

struct NPlayer {
    pub player_name: String,
    pub player_data: LinkOne<NPlayerData>,
    pub identity: LinkOne<NPlayerIdentity>,
    pub active_match: LinkOne<NMatch>,
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
    pub color: LinkOne<NHouseColor>,
    pub ability_magic: LinkOne<NAbilityMagic>,
    pub status_magic: LinkOne<NStatusMagic>,
    pub units: LinkMany<NUnit>,
}

struct NHouseColor {
    pub color: HexColor,
}

struct NAbilityMagic {
    pub ability_name: String,
    pub description: LinkOne<NAbilityDescription>,
}

struct NAbilityDescription {
    pub description: String,
    pub effect: LinkOne<NAbilityEffect>,
}

struct NAbilityEffect {
    pub actions: Actions,
}

struct NStatusMagic {
    pub status_name: String,
    pub description: LinkOne<NStatusDescription>,
    pub representation: LinkOne<NRepresentation>,
}

struct NStatusDescription {
    pub description: String,
    pub behavior: LinkOne<NBehavior>,
}

struct NTeam {
    pub houses: LinkMany<NHouse>,
    pub fusions: LinkMany<NFusion>,
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
    pub shop_case: LinkMany<NShopCaseUnit>,
    pub team: LinkOne<NTeam>,
    pub battles: LinkMany<NBattle>,
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
    pub stats: LinkOne<NFusionStats>,
}

struct NUnit {
    pub unit_name: String,
    pub description: LinkOne<NUnitDescription>,
    pub stats: LinkOne<NUnitStats>,
}

struct NUnitDescription {
    pub description: String,
    pub representation: LinkOne<NRepresentation>,
    pub behavior: LinkOne<NBehavior>,
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
