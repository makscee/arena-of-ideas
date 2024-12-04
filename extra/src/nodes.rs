use super::*;
use bevy::{
    ecs::component::*,
    prelude::{debug, BuildChildren, Commands},
};
use include_dir::Dir;

#[derive(Debug, Clone, Copy, Display)]
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

pub trait ContentNode: Default + Component + Sized {
    fn kind(&self) -> ContentKind;
    fn get_var(&self, var: VarName) -> Option<VarValue>;
    fn inject_data(&mut self, data: &str);
    fn get_data(&self) -> String;
    fn from_data(data: &str) -> Self {
        let mut s = Self::default();
        s.inject_data(data);
        s
    }
    fn from_dir(path: String, dir: &Dir) -> Option<Self>;
    fn unpack(self, entity: Entity, commands: &mut Commands);
}

#[content_node]
pub struct House {
    name: String,
    color: Option<HouseColor>,
    abilities: Vec<Ability>,
}

#[content_node]
pub struct HouseColor {
    pub color: String,
}

#[content_node]
pub struct Ability {
    pub name: String,
    pub description: Option<AbilityDescription>,
    // pub actions: Vec<AbilityEffect>,
    // pub statuses: Vec<Status>,
    pub units: Vec<Unit>,
}

#[content_node]
pub struct AbilityDescription {
    pub data: String,
}

#[content_node]
pub struct AbilityEffect {
    pub data: String,
}

// #[content_node]
// pub struct Status {
//     pub name: String,
//     pub description: Option<StatusDescription>,
// }

// #[content_node]
// pub struct StatusDescription {
//     pub description: String,
//     pub trigger: Option<StatusTrigger>,
// }

// #[content_node]
// pub struct StatusTrigger {
//     pub data: String,
// }

// #[content_node]
// pub struct Summon {
//     pub name: String,
//     pub stats: Option<UnitStats>,
//     pub representation: Option<UnitRepresentation>,
// }

#[content_node]
pub struct Unit {
    pub name: String,
    pub stats: Option<UnitStats>,
    pub description: Option<UnitDescription>,
    // pub representation: Option<UnitRepresentation>,
}

#[content_node]
pub struct UnitStats {
    pub pwr: i32,
    pub hp: i32,
}

#[content_node]
pub struct UnitDescription {
    pub description: String,
    pub trigger: Option<UnitTrigger>,
}

#[content_node]
pub struct UnitTrigger {
    pub trigger: Trigger,
}

// #[content_node]
// pub struct UnitRepresentation {
//     pub data: String,
// }
