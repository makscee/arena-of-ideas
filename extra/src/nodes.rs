use super::*;
use bevy::ecs::component::*;
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
    fn from_dir(path: String, dir: &Dir) -> Option<Self>;
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

// impl Ability {
//     pub fn from_dir(path: String, dir: &Dir) -> Option<Self> {
//         let mut s = Self::default();

//         s.units = dir
//             .get_dir(format!("{path}/units"))
//             .into_iter()
//             .flat_map(|d| d.dirs())
//             .filter_map(|d| Unit::from_dir(d.path().to_string_lossy().to_string(), d))
//             .collect_vec();
//         Some(s)
//     }
// }

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

// impl Unit {
//     pub fn from_dir(path: String, dir: &Dir) -> Option<Self> {
//         let data = &format!("\"{}\"", dir.path().file_name()?.to_str()?);
//         let mut s = Self::from_data(data);
//         s.description = UnitDescription::from_dir(format!("{path}/description"), dir);
//         Some(s)
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
//     pub fn from_dir(path: String, dir: &Dir) -> Option<Self> {
//         let dir = dir.get_dir(&path)?;
//         let data = dir.get_file(format!("{path}/data.ron"))?.contents_utf8()?;
//         let mut s = Self::from_data(data);
//         s.trigger = UnitTrigger::from_dir(format!("{path}/trigger"), dir);
//         Some(s)
//     }
// }

#[content_node]
pub struct UnitTrigger {
    pub trigger: Trigger,
}

// impl UnitTrigger {
//     pub fn from_dir(path: String, dir: &Dir) -> Option<Self> {
//         let data = dir.get_file(format!("{path}.ron"))?.contents_utf8()?;
//         let mut s = Self::from_data(data);
//         Some(s)
//     }
// }

// #[content_node]
// pub struct UnitRepresentation {
//     pub data: String,
// }
