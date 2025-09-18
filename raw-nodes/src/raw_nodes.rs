use super::*;

#[node(system)]
struct NArena {
    pub floor_pools: NodeParts<Child, NFloorPool>,
    pub floor_bosses: NodeParts<Child, NFloorBoss>,
}

#[node(system)]
struct NFloorPool {
    pub floor: i32,
    pub teams: NodeParts<Child, NTeam>,
}

#[node(system)]
struct NFloorBoss {
    pub floor: i32,
    pub team: NodePart<Child, NTeam>,
}

#[node(system)]
struct NPlayer {
    pub player_name: String,
    pub player_data: NodePart<Parent, NPlayerData>,
    pub identity: NodePart<Parent, NPlayerIdentity>,
    pub active_match: NodePart<Parent, NMatch>,
}

#[node(system)]
struct NPlayerData {
    pub pass_hash: Option<String>,
    pub online: bool,
    pub last_login: u64,
}

#[node(system)]
struct NPlayerIdentity {
    pub data: Option<String>,
}

#[node(named)]
struct NHouse {
    pub house_name: String,
    #[link_cardinality(one_to_one)]
    pub color: NodePart<Parent, NHouseColor>,
    #[link_cardinality(one_to_one)]
    pub ability: NodePart<Parent, NAbilityMagic>,
    #[link_cardinality(one_to_one)]
    pub status: NodePart<Parent, NStatusMagic>,
    #[link_cardinality(one_to_many)]
    pub units: NodeParts<Child, NUnit>,
}

#[node(content)]
struct NHouseColor {
    pub color: HexColor,
}

#[node(named)]
struct NAbilityMagic {
    pub ability_name: String,
    #[link_cardinality(one_to_one)]
    pub description: NodePart<Parent, NAbilityDescription>,
}

#[node(content)]
struct NAbilityDescription {
    pub description: String,
    #[link_cardinality(one_to_one)]
    pub effect: NodePart<Parent, NAbilityEffect>,
}

#[node(content)]
struct NAbilityEffect {
    pub actions: Vec<Action>,
}

#[node(named)]
struct NStatusMagic {
    pub status_name: String,
    #[link_cardinality(one_to_one)]
    pub description: NodePart<Parent, NStatusDescription>,
    #[link_cardinality(one_to_one)]
    pub representation: NodePart<Parent, NStatusRepresentation>,
}

#[node(content)]
struct NStatusDescription {
    pub description: String,
    #[link_cardinality(one_to_one)]
    pub behavior: NodePart<Parent, NStatusBehavior>,
}

#[node(content)]
struct NStatusBehavior {
    pub reactions: Vec<Reaction>,
}

#[node(content)]
struct NStatusRepresentation {
    pub material: Material,
}

#[node(system)]
struct NTeam {
    pub houses: NodeParts<Child, NHouse>,
    pub fusions: NodeParts<Child, NFusion>,
}

#[node(system)]
struct NBattle {
    pub team_left: u64,
    pub team_right: u64,
    pub ts: u64,
    pub hash: u64,
    pub result: Option<bool>,
}

#[node(system)]
struct NMatch {
    pub g: i32,
    pub floor: i32,
    pub lives: i32,
    pub active: bool,
    pub shop_offers: Vec<ShopOffer>,
    pub team: NodePart<Child, NTeam>,
    pub battles: NodeParts<Child, NBattle>,
}

#[node(system)]
struct NFusion {
    pub slots: NodeParts<Parent, NFusionSlot>,
    pub trigger_unit: u64,
    pub index: i32,
    pub pwr: i32,
    pub hp: i32,
    pub dmg: i32,
    pub actions_limit: i32,
}

#[node(system)]
struct NFusionSlot {
    pub index: i32,
    pub actions: UnitActionRange,
    pub unit: NodePart<Parent, NUnit>,
}

#[node(named)]
struct NUnit {
    pub unit_name: String,
    #[link_cardinality(one_to_one)]
    pub description: NodePart<Parent, NUnitDescription>,
    #[link_cardinality(many_to_one)]
    pub stats: NodePart<Parent, NUnitStats>,
    #[link_cardinality(one_to_one)]
    pub state: NodePart<Parent, NUnitState>,
}

#[node(content)]
struct NUnitDescription {
    pub description: String,
    pub magic_type: MagicType,
    pub trigger: Trigger,
    #[link_cardinality(one_to_one)]
    pub representation: NodePart<Parent, NUnitRepresentation>,
    #[link_cardinality(one_to_one)]
    pub behavior: NodePart<Parent, NUnitBehavior>,
}

#[node(content)]
struct NUnitStats {
    pub pwr: i32,
    pub hp: i32,
}

#[node(system)]
struct NUnitState {
    pub stacks: i32,
}

#[node(content)]
struct NUnitBehavior {
    pub reaction: Reaction,
    pub magic_type: MagicType,
}

#[node(content)]
struct NUnitRepresentation {
    pub material: Material,
}
