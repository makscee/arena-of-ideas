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
    fn entity(&self) -> Entity;
    fn get_entity(&self) -> Option<Entity>;
    fn from_dir(path: String, dir: &Dir) -> Option<Self>;
    fn to_dir(&self, path: String) -> DirEntry;
    fn from_strings(i: usize, strings: &Vec<String>) -> Option<Self>;
    fn to_strings(&self, parent: usize, field: &str, strings: &mut Vec<String>);
    fn to_strings_root(&self) -> Vec<String> {
        let mut strings = Vec::default();
        self.to_strings(0, "_", &mut strings);
        strings
    }
    fn from_table(domain: NodeDomain, id: u64) -> Option<Self>;
    fn pack(entity: Entity, world: &World) -> Option<Self>;
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
    fn collect_units_vec<'a>(&'a self, vec: &mut Vec<&'a Unit>);
    fn collect_units<'a>(&'a self) -> Vec<&'a Unit> {
        let mut vec: Vec<&Unit> = default();
        self.collect_units_vec(&mut vec);
        vec
    }
}

pub trait NodeDomainExt {
    fn find_by_key(self, key: &String) -> Option<TNode>;
    fn filter_by_kind(self, kind: NodeKind) -> Vec<TNode>;
}

impl NodeDomainExt for NodeDomain {
    fn find_by_key(self, key: &String) -> Option<TNode> {
        match self {
            NodeDomain::World => cn().db.nodes_world().key().find(key),
            NodeDomain::Match => cn().db.nodes_match().key().find(key),
            NodeDomain::Core => cn().db.nodes_core().key().find(key),
        }
    }
    fn filter_by_kind(self, kind: NodeKind) -> Vec<TNode> {
        let kind = kind.to_string();
        match self {
            NodeDomain::World => cn()
                .db
                .nodes_world()
                .iter()
                .filter(|d| d.kind == kind)
                .collect(),
            NodeDomain::Match => cn()
                .db
                .nodes_match()
                .iter()
                .filter(|d| d.kind == kind)
                .collect(),
            NodeDomain::Core => cn()
                .db
                .nodes_core()
                .iter()
                .filter(|d| d.kind == kind)
                .collect(),
        }
    }
}

impl ToCstr for NodeKind {
    fn cstr(&self) -> Cstr {
        self.to_string()
    }
}

trait OnUnpack {
    fn on_unpack(self, entity: Entity, commands: &mut Commands);
}

impl OnUnpack for NodeKind {
    fn on_unpack(self, entity: Entity, commands: &mut Commands) {
        let mut entity_commands = commands.entity(entity);
        match self {
            NodeKind::House => {
                entity_commands.insert(NodeState::new_with(VarName::visible, false.into()));
            }
            NodeKind::Ability | NodeKind::Fusion | NodeKind::Representation | NodeKind::Status => {
                entity_commands.insert(NodeState::default());
            }
            _ => {}
        };
        entity_commands.insert((TransformBundle::default(), VisibilityBundle::default()));

        let mut child = || commands.spawn_empty().set_parent(entity).id();
        match self {
            NodeKind::Hero => hero_rep().clone().unpack(child(), commands),
            NodeKind::Fusion => unit_rep().clone().unpack(entity, commands),
            NodeKind::Status => status_rep().clone().unpack(child(), commands),
            _ => {}
        }
    }
}
