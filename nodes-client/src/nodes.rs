use std::fmt::Debug;

use assets::{hero_rep, unit_rep};

macro_schema::nodes!();

#[bevy_trait_query::queryable]
pub trait GetVar: GetNodeKind {
    fn get_var(&self, var: VarName) -> Option<VarValue>;
    fn set_var(&mut self, var: VarName, value: VarValue);
    fn get_all_vars(&self) -> Vec<(VarName, VarValue)>;
}

pub trait Node: Default + Component + Sized + GetVar + Show + Debug {
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
        let entity = self.entity().expect("Node not linked to world");
        Self::collect_children_entity(entity, context)
    }
    fn ui(&self, depth: usize, context: &Context, ui: &mut Ui);
}

trait OnUnpack {
    fn on_unpack(self, entity: Entity, commands: &mut Commands);
}

impl OnUnpack for NodeKind {
    fn on_unpack(self, entity: Entity, commands: &mut Commands) {
        commands.entity(entity).insert((
            TransformBundle::default(),
            VisibilityBundle::default(),
            NodeState::default(),
        ));

        let entity = commands.spawn_empty().set_parent(entity).id();
        match self {
            NodeKind::Hero => hero_rep().clone().unpack(entity, commands),
            NodeKind::Unit => unit_rep().clone().unpack(entity, commands),
            _ => {}
        }
    }
}
