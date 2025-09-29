use super::*;

#[node]
struct NArena {
    pub floor_pools: Owned<Vec<NFloorPool>>,
    pub floor_bosses: Owned<Vec<NFloorBoss>>,
}

#[node]
struct NFloorPool {
    pub floor: i32,
    pub teams: Owned<Vec<NTeam>>,
}

#[node]
struct NFloorBoss {
    pub floor: i32,
    pub team: Component<NTeam>,
}

#[node]
struct NPlayer {
    pub player_name: String,
    pub player_data: Component<NPlayerData>,
    pub identity: Component<NPlayerIdentity>,
    pub active_match: Component<NMatch>,
}

#[node]
struct NPlayerData {
    pub pass_hash: Option<String>,
    pub online: bool,
    pub last_login: u64,
}

#[node]
struct NPlayerIdentity {
    pub data: Option<String>,
}

#[node(content, name = House)]
struct NHouse {
    pub house_name: String,
    pub color: Component<NHouseColor>,
    pub ability: Component<NAbilityMagic>,
    pub status: Component<NStatusMagic>,
    pub units: Owned<Vec<NUnit>>,
}

#[node(content)]
struct NHouseColor {
    pub color: HexColor,
}

#[node(content, name = Ability)]
struct NAbilityMagic {
    pub ability_name: String,
    pub description: Component<NAbilityDescription>,
}

#[node(content)]
struct NAbilityDescription {
    pub description: String,
    pub effect: Component<NAbilityEffect>,
}

#[node(content)]
struct NAbilityEffect {
    pub actions: Vec<Action>,
}

#[node(content, name = Status)]
struct NStatusMagic {
    pub status_name: String,
    pub description: Component<NStatusDescription>,
    pub representation: Component<NStatusRepresentation>,
}

#[node(content)]
struct NStatusDescription {
    pub description: String,
    pub behavior: Component<NStatusBehavior>,
}

#[node(content)]
struct NStatusBehavior {
    pub reactions: Vec<Reaction>,
}

#[node(content)]
struct NStatusRepresentation {
    pub material: Material,
}

#[node]
struct NTeam {
    pub houses: Owned<Vec<NHouse>>,
    pub fusions: Owned<Vec<NFusion>>,
}

#[node]
struct NBattle {
    pub team_left: u64,
    pub team_right: u64,
    pub ts: u64,
    pub hash: u64,
    pub result: Option<bool>,
}

#[node]
struct NMatch {
    pub g: i32,
    pub floor: i32,
    pub lives: i32,
    pub active: bool,
    pub shop_offers: Vec<ShopOffer>,
    pub team: Component<NTeam>,
    pub battles: Owned<Vec<NBattle>>,
}

#[node]
struct NFusion {
    pub slots: Owned<Vec<NFusionSlot>>,
    pub trigger_unit: u64,
    pub index: i32,
    pub pwr: i32,
    pub hp: i32,
    pub dmg: i32,
    pub actions_limit: i32,
}

#[node]
struct NFusionSlot {
    pub index: i32,
    pub actions: UnitActionRange,
    pub unit: Ref<Option<NUnit>>,
}

#[node(content)]
struct NUnit {
    pub unit_name: String,
    pub description: Component<NUnitDescription>,
    pub stats: Component<NUnitStats>,
    pub state: Component<NUnitState>,
}

#[node(content)]
struct NUnitDescription {
    pub description: String,
    pub magic_type: MagicType,
    pub trigger: Trigger,
    pub representation: Component<NUnitRepresentation>,
    pub behavior: Component<NUnitBehavior>,
}

#[node(content)]
struct NUnitStats {
    pub pwr: i32,
    pub hp: i32,
}

#[node]
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
