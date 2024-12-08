use super::*;
use bevy::{math::vec2, prelude::Query};
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
    pub fn get_var_e(var: VarName, entity: Entity, state: &StateQuery) -> Option<VarValue> {
        let v = state
            .get_state(entity)
            .and_then(|s| s.vars.get(&var).cloned());
        if v.is_some() {
            v
        } else {
            if let Some(p) = state.get_parent(entity) {
                Self::get_var_e(var, p.get(), state)
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

#[node]
pub struct Representation {
    pub material: RMaterial,
    pub children: Vec<Box<Representation>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct RMaterial {
    pub t: MaterialType,
    #[serde(default)]
    pub count: u32,
    #[serde(default)]
    pub modifiers: Vec<RModifier>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum RModifier {
    Color(Expression),
    Offset(Expression),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum MaterialType {
    Shape {
        shape: Shape,
        #[serde(default)]
        modifiers: Vec<ShapeModifier>,
    },
    Text {
        text: Expression,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Shape {
    Rectangle { size: Expression },
    Circle { radius: Expression },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ShapeModifier {
    Rotation(Expression),
    Scale(Expression),
    Color(Expression),
    Hollow(Expression),
    Thickness(Expression),
    Roundness(Expression),
    Alpha(Expression),
}

impl Default for Shape {
    fn default() -> Self {
        Shape::Rectangle {
            size: Expression::V(vec2(1.0, 1.0).into()),
        }
    }
}
impl Default for MaterialType {
    fn default() -> Self {
        Self::Shape {
            shape: default(),
            modifiers: default(),
        }
    }
}
impl Default for ShapeModifier {
    fn default() -> Self {
        Self::Rotation(Expression::Zero)
    }
}
