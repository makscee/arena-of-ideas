use super::*;

struct All {
    pub name: String,
    pub players: NodeChildren<Player>,
    pub core: NodeChildren<House>,
    pub incubator: NodeComponent<Incubator>,
}

struct Incubator {
    pub name: String,
    pub houses: NodeChildren<House>,
    pub house_colors: NodeChildren<HouseColor>,
    pub units: NodeChildren<Unit>,
    pub unit_descriptions: NodeChildren<UnitDescription>,
    pub unit_stats: NodeChildren<UnitStats>,
    pub action_abilities: NodeChildren<ActionAbility>,
    pub action_ability_descriptions: NodeChildren<ActionAbilityDescription>,
    pub ability_effects: NodeChildren<AbilityEffect>,
    pub status_abilities: NodeChildren<StatusAbility>,
    pub status_ability_descriptions: NodeChildren<StatusAbilityDescription>,
    pub representations: NodeChildren<Representation>,
    pub reactions: NodeChildren<Behavior>,
}

struct Player {
    pub name: String,
    pub player_data: NodeComponent<PlayerData>,
    pub identity: NodeComponent<PlayerIdentity>,
    pub active_match: NodeComponent<Match>,
}

struct PlayerData {
    pub pass_hash: Option<String>,
    pub online: bool,
    pub last_login: u64,
}

struct PlayerIdentity {
    pub data: Option<String>,
}

struct House {
    pub name: String,
    pub color: NodeComponent<HouseColor>,
    pub action_abilities: NodeComponent<ActionAbility>,
    pub status_abilities: NodeComponent<StatusAbility>,
    pub units: NodeChildren<Unit>,
}

struct HouseColor {
    pub color: HexColor,
}

struct ActionAbility {
    pub name: String,
    pub description: NodeComponent<ActionAbilityDescription>,
}

struct ActionAbilityDescription {
    pub description: String,
    pub effect: NodeComponent<AbilityEffect>,
}

struct AbilityEffect {
    pub actions: Actions,
}

struct StatusAbility {
    pub name: String,
    pub description: NodeComponent<StatusAbilityDescription>,
    pub representation: NodeComponent<Representation>,
}

struct StatusAbilityDescription {
    pub description: String,
    pub reaction: NodeComponent<Behavior>,
}

struct Team {
    pub name: String,
    pub houses: NodeChildren<House>,
    pub fusions: NodeChildren<Fusion>,
}

struct Match {
    pub g: i32,
    pub shop_case: NodeChildren<ShopCaseUnit>,
    pub team: NodeComponent<Team>,
}

struct ShopCaseUnit {
    pub unit: u64,
    pub price: i32,
    pub sold: bool,
}

struct Fusion {
    pub units: Vec<String>,
    pub triggers: Vec<(UnitTriggerRef, Vec<UnitActionRef>)>,
    pub slot: i32,
}

struct Unit {
    pub name: String,
    pub description: NodeComponent<UnitDescription>,
}

struct UnitDescription {
    pub description: String,
    pub representation: NodeComponent<Representation>,
    pub stats: NodeComponent<UnitStats>,
    pub reaction: NodeComponent<Behavior>,
}

struct UnitStats {
    pub pwr: i32,
    pub hp: i32,
    pub dmg: i32,
}

struct Behavior {
    pub triggers: Vec<Reaction>,
}

struct Representation {
    pub material: Material,
}
