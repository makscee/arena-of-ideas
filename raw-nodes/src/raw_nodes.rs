use super::*;

struct NArena {
    pub floor_pools: NodeParts<Child, NFloorPool>,
    pub floor_bosses: NodeParts<Child, NFloorBoss>,
}

struct NFloorPool {
    pub floor: i32,
    pub teams: NodeParts<Child, NTeam>,
}

struct NFloorBoss {
    pub floor: i32,
    pub team: NodePart<Child, NTeam>,
}

struct NPlayer {
    pub player_name: String,
    pub player_data: NodePart<Parent, NPlayerData>,
    pub identity: NodePart<Parent, NPlayerIdentity>,
    pub active_match: NodePart<Parent, NMatch>,
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
    pub color: NodePart<Parent, NHouseColor>,
    pub action: NodePart<Parent, NActionAbility>,
    pub status: NodePart<Parent, NStatusAbility>,
    pub units: NodeParts<Child, NUnit>,
}

struct NHouseColor {
    pub color: HexColor,
}

struct NActionAbility {
    pub ability_name: String,
    pub description: NodePart<Parent, NActionDescription>,
}

struct NActionDescription {
    pub description: String,
    pub effect: NodePart<Parent, NActionEffect>,
}

struct NActionEffect {
    pub actions: Vec<Action>,
}

struct NStatusAbility {
    pub status_name: String,
    pub description: NodePart<Parent, NStatusDescription>,
    pub representation: NodePart<Parent, NStatusRepresentation>,
}

struct NStatusDescription {
    pub description: String,
    pub behavior: NodePart<Parent, NStatusBehavior>,
}

struct NStatusBehavior {
    pub reactions: Vec<Reaction>,
}

struct NStatusRepresentation {
    pub material: Material,
}

struct NTeam {
    pub houses: NodeParts<Child, NHouse>,
    pub fusions: NodeParts<Child, NFusion>,
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
    pub lives: i32,
    pub active: bool,
    pub shop_offers: Vec<ShopOffer>,
    pub team: NodePart<Child, NTeam>,
    pub battles: NodeParts<Child, NBattle>,
}

struct NFusion {
    pub slots: NodeParts<Parent, NFusionSlot>,
    pub trigger: UnitTriggerRef,
    pub index: i32,
    pub pwr: i32,
    pub hp: i32,
    pub dmg: i32,
    pub actions_limit: i32,
}

struct NFusionSlot {
    pub index: i32,
    pub actions: UnitActionRange,
    pub unit: NodePart<Parent, NUnit>,
}

struct NUnit {
    pub unit_name: String,
    pub description: NodePart<Parent, NUnitDescription>,
    pub stats: NodePart<Parent, NUnitStats>,
    pub state: NodePart<Parent, NUnitState>,
}

struct NUnitDescription {
    pub description: String,
    pub representation: NodePart<Parent, NUnitRepresentation>,
    pub behavior: NodePart<Parent, NUnitBehavior>,
}

struct NUnitStats {
    pub pwr: i32,
    pub hp: i32,
}

struct NUnitState {
    pub stacks: i32,
}

struct NUnitBehavior {
    pub reactions: Vec<Reaction>,
}

struct NUnitRepresentation {
    pub material: Material,
}
