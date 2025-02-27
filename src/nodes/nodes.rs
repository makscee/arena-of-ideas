use include_dir::{DirEntry, File};
use macro_client::*;
use std::fmt::Debug;

macro_schema::nodes!();

#[bevy_trait_query::queryable]
pub trait GetVar: GetNodeKind + Debug {
    fn get_var(&self, var: VarName) -> Option<VarValue>;
    fn set_var(&mut self, var: VarName, value: VarValue);
    fn get_vars(&self) -> Vec<(VarName, VarValue)>;
    fn get_all_vars(&self) -> Vec<(VarName, VarValue)>;
}

pub trait Node: Default + Component + Sized + GetVar + Show + Debug {
    fn id(&self) -> u64;
    fn set_id(&mut self, id: u64);
    fn entity(&self) -> Entity;
    fn get_entity(&self) -> Option<Entity>;
    fn clear_entities(&mut self);
    fn from_dir(path: String, dir: &Dir) -> Option<Self>;
    fn to_dir(&self, path: String) -> DirEntry;
    fn from_strings(i: usize, strings: &Vec<String>) -> Option<Self>;
    fn to_strings(&self, parent: usize, field: &str, strings: &mut Vec<String>);
    fn to_strings_root(&self) -> Vec<String> {
        let mut strings = Vec::default();
        self.to_strings(0, "_", &mut strings);
        strings
    }
    fn load_recursive(id: u64) -> Option<Self>;
    fn pack(entity: Entity, world: &World) -> Option<Self>;
    fn unpack(self, entity: Entity, world: &mut World);
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
        let entity = self.get_entity().expect("Node not linked to world");
        Self::find_up_entity::<T>(entity, world)
    }
    fn collect_children_entity<'a, T: Component>(
        entity: Entity,
        context: &'a Context,
    ) -> Vec<(Entity, &'a T)> {
        context
            .get_children(entity)
            .into_iter()
            .filter_map(|e| context.get_component::<T>(e).map(|c| (e, c)))
            .collect_vec()
    }
    fn collect_children<'a, T: Component>(&self, context: &'a Context) -> Vec<(Entity, &'a T)> {
        let entity = self.get_entity().expect("Node not linked to world");
        Self::collect_children_entity(entity, context)
    }
}

pub trait NodeExt: Sized {
    fn get(id: u64) -> Option<Self>;
}
impl<T> NodeExt for T
where
    T: Node + GetNodeKind + GetNodeKindSelf,
{
    fn get(id: u64) -> Option<Self> {
        let kind = Self::kind_s();
        cn().db
            .tnodes()
            .key()
            .find(&kind.key(id))
            .map(|d| d.to_node())
    }
}

impl TNode {
    pub fn to_node<T: Node>(self) -> T {
        let mut d = T::default();
        d.inject_data(&self.data);
        d.set_id(self.id);
        d
    }
}

#[derive(Resource, Default)]
pub struct IdEntityLinks {
    map: HashMap<u64, Entity>,
}

#[derive(Resource, Default)]
pub struct NameEntityLinks {
    map: HashMap<String, Entity>,
}

pub trait WorldNodeExt {
    fn add_id_link(&mut self, id: u64, entity: Entity);
    fn get_id_link(&self, id: u64) -> Option<Entity>;
    fn add_name_link(&mut self, name: String, entity: Entity);
    fn get_name_link(&self, name: &str) -> Option<Entity>;
}

impl WorldNodeExt for World {
    fn add_id_link(&mut self, id: u64, entity: Entity) {
        self.get_resource_or_insert_with::<IdEntityLinks>(|| default())
            .map
            .insert(id, entity);
    }
    fn get_id_link(&self, id: u64) -> Option<Entity> {
        self.get_resource::<IdEntityLinks>()
            .and_then(|r| r.map.get(&id))
            .copied()
    }
    fn add_name_link(&mut self, name: String, entity: Entity) {
        self.get_resource_or_insert_with::<NameEntityLinks>(|| default())
            .map
            .insert(name, entity);
    }
    fn get_name_link(&self, name: &str) -> Option<Entity> {
        self.get_resource::<NameEntityLinks>()
            .and_then(|r| r.map.get(name))
            .copied()
    }
}

impl ToCstr for NodeKind {
    fn cstr(&self) -> Cstr {
        self.to_string()
    }
}

trait OnUnpack {
    fn on_unpack(self, entity: Entity, world: &mut World);
}

impl OnUnpack for NodeKind {
    fn on_unpack(self, entity: Entity, world: &mut World) {
        let vars = self.get_vars(entity, world);
        let mut emut = world.entity_mut(entity);
        let mut ns = if let Some(ns) = emut.get_mut::<NodeState>() {
            ns
        } else {
            emut.insert(NodeState::default())
                .get_mut::<NodeState>()
                .unwrap()
        };
        ns.init_vars(vars);
        match self {
            NodeKind::House => {
                ns.init(VarName::visible, false.into());
            }
            _ => {}
        };
        emut.insert((Transform::default(), Visibility::default()));

        let mut child = || world.spawn_empty().set_parent(entity).id();
        match self {
            NodeKind::Fusion => {
                unit_rep().clone().unpack(entity, world);
                Fusion::init(entity, world).log();
            }
            NodeKind::StatusAbility => status_rep().clone().unpack(child(), world),
            _ => {}
        }
    }
}
