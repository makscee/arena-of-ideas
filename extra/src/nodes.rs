use super::*;

struct House {
    name: String,
    color: HouseColor,
    abilities: Vec<Ability>,
}

struct HouseColor {
    data: String,
}

#[derive(ContentNode)]
pub struct Ability {
    pub description: String,
    // actions: Vec<AbilityAction>,
    pub units: Vec<Unit>,
}

struct AbilityAction {
    effect: String,
}

#[derive(ContentNode)]
pub struct Unit {
    pub name: String,
    pub stats: UnitStats,
    pub description: UnitDescription,
}

#[derive(ContentNode)]
pub struct UnitStats {
    pub data: String,
}

#[derive(ContentNode)]
pub struct UnitDescription {
    pub text: String,
}

#[derive(Debug)]
pub enum ContentKind {
    Ability,
    Unit,
    UnitDescription,
    UnitStats,
}

pub trait ContentNode {
    fn kind(&self) -> ContentKind;
    fn data(&self) -> &String;
    fn data_mut(&mut self) -> &mut String;
    fn links(&self, f: fn(&dyn ContentNode));
    fn walk(&self, f: fn(&dyn ContentNode));
}
