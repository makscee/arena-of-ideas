use super::*;

struct All {
    pub name: String,
    pub players: NodeChildren<Player>,
    pub core: NodeChildren<House>,
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
    pub action_abilities: NodeChildren<ActionAbility>,
    pub status_abilities: NodeChildren<StatusAbility>,
    pub units: NodeChildren<Unit>,
}

struct HouseColor {
    pub color: String,
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
    pub reaction: NodeComponent<Reaction>,
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
    pub unit: String,
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
    pub reaction: NodeComponent<Reaction>,
}

struct UnitStats {
    pub pwr: i32,
    pub hp: i32,
    pub dmg: i32,
}

struct Reaction {
    pub triggers: Vec<(Trigger, Actions)>,
}

struct Representation {
    pub material: Material,
}
