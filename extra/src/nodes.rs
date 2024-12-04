use super::*;
use bevy::{
    ecs::component::*,
    prelude::{debug, BuildChildren, Commands, Parent, TransformBundle, VisibilityBundle, World},
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
    Representation,
    UnitTrigger,
}

pub trait ContentNode: Default + Component + Sized {
    fn kind(&self) -> ContentKind;
    fn entity(&self) -> Option<Entity>;
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
    fn find_up_entity<T: ContentNode>(entity: Entity, world: &World) -> Option<&T> {
        let r = world.get::<T>(entity);
        if r.is_some() {
            r
        } else {
            let p = world.get::<Parent>(entity);
            if let Some(p) = p {
                Self::find_up_entity(p.get(), world)
            } else {
                None
            }
        }
    }
    fn find_up<'a, T: ContentNode>(&self, world: &'a World) -> Option<&'a T> {
        let entity = self.entity().expect("Node not linked to world");
        Self::find_up_entity::<T>(entity, world)
    }
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
    pub representation: Option<Representation>,
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

#[content_node(OnUnpack)]
pub struct Representation {
    pub material: RepresentationMaterial,
    pub count: u32,
    pub child: Option<Box<Representation>>,
}

impl Representation {
    fn on_unpack(&self, entity: Entity, commands: &mut Commands) {
        debug!("on unpack called");
        self.material.unpack(entity, commands);
    }
}
