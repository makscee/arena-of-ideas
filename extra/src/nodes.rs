use super::*;
use include_dir::Dir;

#[derive(Debug, Clone, Copy, Display, EnumIter)]
pub enum NodeKind {
    House,
    HouseColor,
    Ability,
    AbilityDescription,
    AbilityEffect,
    Unit,
    UnitDescription,
    UnitStats,
    Representation,
    UnitTrigger,
}

impl NodeKind {
    pub fn register(self, app: &mut App) {
        use bevy_trait_query::RegisterExt;
        match self {
            NodeKind::House => app.register_component_as::<dyn GetVar, House>(),
            NodeKind::HouseColor => app.register_component_as::<dyn GetVar, HouseColor>(),
            NodeKind::Ability => app.register_component_as::<dyn GetVar, Ability>(),
            NodeKind::AbilityDescription => {
                app.register_component_as::<dyn GetVar, AbilityDescription>()
            }
            NodeKind::AbilityEffect => app.register_component_as::<dyn GetVar, AbilityEffect>(),
            NodeKind::Unit => app.register_component_as::<dyn GetVar, Unit>(),
            NodeKind::UnitDescription => app.register_component_as::<dyn GetVar, UnitDescription>(),
            NodeKind::UnitStats => app.register_component_as::<dyn GetVar, UnitStats>(),
            NodeKind::Representation => app.register_component_as::<dyn GetVar, Representation>(),
            NodeKind::UnitTrigger => app.register_component_as::<dyn GetVar, UnitTrigger>(),
        };
    }
}

#[bevy_trait_query::queryable]
pub trait GetVar {
    fn get_var(&self, var: VarName) -> Option<VarValue>;
    fn get_all_vars(&self) -> Vec<(VarName, VarValue)>;
}

#[derive(Component, Reflect)]
pub struct NodeState {
    pub vars: HashMap<VarName, VarValue>,
}

impl NodeState {
    pub fn get_var_e(var: VarName, entity: Entity, world: &World) -> Option<VarValue> {
        let v = world
            .get::<Self>(entity)
            .and_then(|s| s.vars.get(&var).cloned());
        if v.is_some() {
            v
        } else {
            if let Some(p) = world.get::<Parent>(entity) {
                Self::get_var_e(var, p.get(), world)
            } else {
                None
            }
        }
    }
}

pub trait Node: Default + Component + Sized + GetVar {
    fn kind(&self) -> NodeKind;
    fn entity(&self) -> Option<Entity>;
    fn inject_data(&mut self, data: &str);
    fn get_data(&self) -> String;
    fn from_data(data: &str) -> Self {
        let mut s = Self::default();
        s.inject_data(data);
        s
    }
    fn from_dir(path: String, dir: &Dir) -> Option<Self>;
    fn unpack(self, entity: Entity, commands: &mut Commands);
    fn find_up_entity<T: Component>(entity: Entity, world: &World) -> Option<&T> {
        let r = world.get::<T>(entity);
        if r.is_some() {
            r
        } else {
            if let Some(p) = world.get::<Parent>(entity) {
                Self::find_up_entity(p.get(), world)
            } else {
                None
            }
        }
    }
    fn find_up<'a, T: Component>(&self, world: &'a World) -> Option<&'a T> {
        let entity = self.entity().expect("Node not linked to world");
        Self::find_up_entity::<T>(entity, world)
    }
}

#[node]
pub struct House {
    name: String,
    color: Option<HouseColor>,
    abilities: Vec<Ability>,
}

#[node]
pub struct HouseColor {
    pub color: String,
}

#[node]
pub struct Ability {
    pub name: String,
    pub description: Option<AbilityDescription>,
    // pub actions: Vec<AbilityEffect>,
    // pub statuses: Vec<Status>,
    pub units: Vec<Unit>,
}

#[node]
pub struct AbilityDescription {
    pub data: String,
}

#[node]
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

#[node]
pub struct Unit {
    pub name: String,
    pub stats: Option<UnitStats>,
    pub description: Option<UnitDescription>,
    pub representation: Option<Representation>,
}

#[node]
pub struct UnitStats {
    pub pwr: i32,
    pub hp: i32,
}

#[node]
pub struct UnitDescription {
    pub description: String,
    pub trigger: Option<UnitTrigger>,
}

#[node]
pub struct UnitTrigger {
    pub trigger: Trigger,
}

#[node(on_unpack)]
pub struct Representation {
    pub material: RepresentationMaterial,
    pub count: u32,
    pub children: Vec<Box<Representation>>,
}

impl Representation {
    fn on_unpack(&self, entity: Entity, commands: &mut Commands) {
        debug!("on unpack called");
        self.material.unpack(entity, commands);
    }
}
