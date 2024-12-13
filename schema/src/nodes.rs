use super::*;

struct House {
    pub name: String,
    pub color: Option<HouseColor>,
    pub abilities: Vec<Ability>,
}

struct HouseColor {
    pub color: String,
}

struct Ability {
    pub name: String,
    pub description: Option<AbilityDescription>,
    pub units: Vec<Unit>,
}

struct AbilityDescription {
    pub data: String,
}

struct AbilityEffect {
    pub data: String,
}

struct Unit {
    pub name: String,
    pub stats: Option<UnitStats>,
    pub description: Option<UnitDescription>,
}

struct UnitStats {
    pub pwr: i32,
    pub hp: i32,
}

struct UnitDescription {
    pub description: String,
}

struct Hero {
    pub name: String,
}
