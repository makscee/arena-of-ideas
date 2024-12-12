use super::*;

#[derive(Debug, Clone, Copy, Display, EnumIter, Reflect, PartialEq, Eq)]
#[node_kinds]
pub enum NodeKind {
    Hero = 0,
    House = 1,
    HouseColor = 2,
    Ability = 3,
    AbilityDescription = 4,
    AbilityEffect = 5,
    Unit = 6,
    UnitDescription = 7,
    UnitStats = 8,
    Representation = 9,
    UnitTrigger = 10,
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

#[derive(Component)]
pub struct NodeState {
    pub vars: HashMap<VarName, VarValue>,
    pub source: HashMap<VarName, NodeKind>,
}

impl NodeState {
    pub fn get_var_state(var: VarName, entity: Entity, state: &StateQuery) -> Option<VarValue> {
        let v = state
            .get_state(entity)
            .and_then(|s| s.vars.get(&var).cloned());
        if v.is_some() {
            v
        } else {
            if let Some(p) = state.get_parent(entity) {
                Self::get_var_state(var, p, state)
            } else {
                None
            }
        }
    }
    pub fn get_var_world(var: VarName, entity: Entity, world: &World) -> Option<VarValue> {
        let v = world
            .get::<NodeState>(entity)
            .and_then(|s| s.vars.get(&var).cloned());
        if v.is_some() {
            v
        } else {
            if let Some(p) = get_parent(entity, world) {
                Self::get_var_world(var, p, world)
            } else {
                None
            }
        }
    }
}

pub trait Node: Default + Component + Sized + GetVar + Show {
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
    fn ui(&self, depth: usize, ui: &mut Ui, world: &World);
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

#[derive(Component)]
pub struct UnitComponent {
    pub entity: Option<Entity>,
    pub name: String,
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

#[node(on_unpack)]
pub struct Hero {
    pub name: String,
    pub representation: Option<Representation>,
}

impl Hero {
    fn on_unpack(&self, entity: Entity, commands: &mut Commands) {
        let entity = commands.spawn_empty().set_parent(entity).id();
        HERO_REP.get().unwrap().clone().unpack(entity, commands);
    }
}
