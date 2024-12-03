use std::path::PathBuf;

use super::*;
use bevy::{ecs::component::*, log::error};
use include_dir::{include_dir, Dir, DirEntry};
use serde::{Deserialize, Serialize};

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

// trait DirExt {
//     fn dir_name(&self) -> &str;
// }

// impl DirExt for Dir {
//     fn dir_name(&self) -> &str {
//         self.path().
//     }
// }

pub trait ContentNode: Default {
    fn kind(&self) -> ContentKind;
    fn get_var(&self, var: VarName) -> Option<VarValue>;
    fn inject_data(&mut self, data: &str);
    fn get_data(&self) -> String;
    fn from_data(data: &str) -> Self {
        let mut s = Self::default();
        s.inject_data(data);
        s
    }
    fn from_entry(dir: Option<&DirEntry>) -> Option<Self>;
    // fn from_dir(dir: &Dir) -> Self;
}

// #[content_node]
// pub struct House {
//     name: String,
//     color: Option<HouseColor>,
//     abilities: Vec<Ability>,
// }

// #[content_node]
// pub struct HouseColor {
//     pub color: String,
// }

// #[derive(ContentNode)]
// pub struct Ability {
//     pub name: String,
//     pub description: Option<AbilityDescription>,
//     pub actions: Vec<AbilityEffect>,
//     pub statuses: Vec<Status>,
//     pub units: Vec<Unit>,
// }

// #[derive(ContentNode)]
// pub struct AbilityDescription {
//     pub data: String,
// }

// #[derive(ContentNode)]
// pub struct AbilityEffect {
//     pub data: String,
// }

// #[derive(ContentNode)]
// pub struct Status {
//     pub name: String,
//     pub description: Option<StatusDescription>,
// }

// #[derive(ContentNode)]
// pub struct StatusDescription {
//     pub description: String,
//     pub trigger: Option<StatusTrigger>,
// }

// #[derive(ContentNode)]
// pub struct StatusTrigger {
//     pub data: String,
// }

// #[derive(ContentNode)]
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

// impl Unit {
//     pub fn from_entry(dir: Option<&DirEntry>) -> Option<Self> {
//         let Some(dir) = dir.and_then(|d| d.as_dir()) else {
//             return None;
//         };
//         let dir_name = dir.path().file_name().unwrap().to_string_lossy();
//         Some(Self {
//             name: dir_name.into(),
//             stats: default(),
//             description: UnitDescription::from_entry(dir.get_entry(dir.path().join("description"))),
//         })
//     }
// }

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

// impl UnitDescription {
//     pub fn from_entry(dir: Option<&DirEntry>) -> Option<Self> {
//         let Some(dir) = dir.and_then(|d| d.as_dir()) else {
//             return None;
//         };
//         dir.get_file(dir.path().join("data.ron"))
//             .and_then(|f| f.contents_utf8())
//             .map(|c| {
//                 let mut s = Self::from_data(c);
//                 s.trigger = UnitTrigger::from_entry(dir.get_entry(dir.path().join("trigger.ron")));
//                 s
//             })
//     }
// }

#[content_node]
pub struct UnitTrigger {
    pub trigger: Trigger,
}

// impl UnitTrigger {
//     pub fn from_entry(dir: Option<&DirEntry>) -> Option<Self> {
//         dir.and_then(|d| d.as_file())
//             .and_then(|f| f.contents_utf8())
//             .map(|f| Self::from_data(f))
//     }
// }

// #[derive(ContentNode)]
// pub struct UnitRepresentation {
//     pub data: String,
// }

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
pub enum VarName {
    hp,
    pwr,
    data,
    name,
    description,
    color,
    lvl,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum Trigger {
    #[default]
    BattleStart,
    TurnEnd,
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
