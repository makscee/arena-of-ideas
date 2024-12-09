use super::*;
use bevy::{color::Color, math::vec2};
use bevy_egui::egui::Ui;
use include_dir::Dir;
use ui::Show;

#[derive(Debug, Clone, Copy, Display, EnumIter, Reflect)]
#[node_kinds]
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

#[bevy_trait_query::queryable]
pub trait GetVar: GetNodeKind {
    fn get_var(&self, var: VarName) -> Option<VarValue>;
    fn set_var(&mut self, var: VarName, value: VarValue);
    fn get_all_vars(&self) -> Vec<(VarName, VarValue)>;
}

pub trait GetNodeKind {
    fn kind(&self) -> NodeKind;
}

#[derive(Component, Reflect)]
pub struct NodeState {
    pub vars: HashMap<VarName, VarValue>,
    pub source: HashMap<VarName, NodeKind>,
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
    fn collect_children_entity<T: Component>(entity: Entity, world: &World) -> Vec<&T> {
        get_children(entity, world)
            .into_iter()
            .filter_map(|c| world.get::<T>(c))
            .collect_vec()
    }
    fn collect_children<'a, T: Component>(&self, world: &'a World) -> Vec<&'a T> {
        let entity = self.entity().expect("Node not linked to world");
        Self::collect_children_entity(entity, world)
    }
    fn show_self(&self, ui: &mut Ui) {
        for (var, value) in self.get_all_vars() {
            value.show(Some(&var.to_string()), ui);
        }
    }
    fn show(&self, ui: &mut Ui, world: &World);
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

#[node(on_unpack)]
pub struct Unit {
    pub name: String,
    pub stats: Option<UnitStats>,
    pub description: Option<UnitDescription>,
    pub representation: Option<Representation>,
}

impl Unit {
    fn on_unpack(&self, entity: Entity, commands: &mut Commands) {
        let entity = commands.spawn_empty().set_parent(entity).id();
        UNIT_REP.get().unwrap().clone().unpack(entity, commands);
    }
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
