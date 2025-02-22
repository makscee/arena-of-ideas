use super::*;

struct House {
    pub name: String,
    pub color: NodeComponent<HouseColor>,
    pub action_abilities: NodeChildren<ActionAbility>,
    pub status_abilities: NodeChildren<StatusAbility>,
}

struct HouseColor {
    pub color: String,
}

struct ActionAbility {
    pub name: String,
    pub description: NodeComponent<ActionAbilityDescription>,
    pub units: NodeChildren<Unit>,
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
    pub units: NodeChildren<Unit>,
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
    pub last_update: u64,
    pub shop_case: NodeChildren<ShopCaseUnit>,
    pub team: NodeComponent<Team>,
}

struct ShopCaseUnit {
    pub unit_id: u64,
    pub price: i32,
    pub sold: bool,
}

struct Fusion {
    pub units: Vec<String>,
    pub triggers: Vec<(UnitTriggerRef, Vec<UnitActionRef>)>,
    pub slot: NodeComponent<UnitSlot>,
}

struct Unit {
    pub name: String,
    pub stats: NodeComponent<UnitStats>,
    pub description: NodeComponent<UnitDescription>,
    pub representation: NodeComponent<Representation>,
}

struct UnitStats {
    pub pwr: i32,
    pub hp: i32,
    pub dmg: i32,
}

struct UnitSlot {
    pub slot: i32,
}

struct UnitDescription {
    pub description: String,
    pub reaction: NodeComponent<Reaction>,
}

struct Reaction {
    pub triggers: Vec<(Trigger, Actions)>,
}

struct Representation {
    pub material: Material,
}
