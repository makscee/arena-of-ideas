use super::*;

#[derive(Debug, Clone, Copy)]
pub enum ContentKind {
    House,
    HouseColor,
    Ability,
    AbilityDescription,
    AbilityEffect,
    Status,
    StatusDescription,
    StatusTrigger,
    Summon,
    Unit,
    UnitDescription,
    UnitStats,
    UnitRepresentation,
    UnitTrigger,
}

pub trait ContentNode {
    fn kind(&self) -> ContentKind;
    fn get_var(&self, var: VarName) -> Option<VarValue>;
    fn walk(&self, f: fn(&dyn ContentNode));
}

#[derive(ContentNode)]
pub struct House {
    name: String,
    color: HouseColor,
    abilities: Vec<Ability>,
}

#[derive(ContentNode)]
pub struct HouseColor {
    pub color: String,
}

#[derive(ContentNode)]
pub struct Ability {
    pub name: String,
    pub description: AbilityDescription,
    pub actions: Vec<AbilityEffect>,
    pub statuses: Vec<Status>,
    pub units: Vec<Unit>,
}

#[derive(ContentNode)]
pub struct AbilityDescription {
    pub data: String,
}

#[derive(ContentNode)]
pub struct AbilityEffect {
    pub data: String,
}

#[derive(ContentNode)]
pub struct Status {
    pub name: String,
    pub description: StatusDescription,
}

#[derive(ContentNode)]
pub struct StatusDescription {
    pub description: String,
    pub trigger: StatusTrigger,
}

#[derive(ContentNode)]
pub struct StatusTrigger {
    pub data: String,
}

#[derive(ContentNode)]
pub struct Summon {
    pub name: String,
    pub stats: UnitStats,
    pub representation: UnitRepresentation,
}

#[derive(ContentNode)]
pub struct Unit {
    pub name: String,
    pub stats: UnitStats,
    pub description: UnitDescription,
    pub representation: UnitRepresentation,
}

#[derive(ContentNode)]
pub struct UnitStats {
    pub hp: i32,
    pub pwr: i32,
}

#[derive(ContentNode)]
pub struct UnitDescription {
    pub description: String,
    pub trigger: UnitTrigger,
}

#[derive(ContentNode)]
pub struct UnitTrigger {
    pub data: String,
}

#[derive(ContentNode)]
pub struct UnitRepresentation {
    pub data: String,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
pub enum VarName {
    hp,
    pwr,
    data,
    name,
    description,
    color,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub enum VarValue {
    i32(i32),
    f32(f32),
    String(String),
}

impl Default for VarValue {
    fn default() -> Self {
        Self::i32(0)
    }
}
