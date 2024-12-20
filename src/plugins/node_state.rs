use bevy::{
    app::PreUpdate,
    math::Vec3Swizzles,
    prelude::{Commands, In},
};

use super::*;

pub struct NodeStatePlugin;

impl Plugin for NodeStatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, Self::collect_vars);
    }
}

impl NodeStatePlugin {
    fn collect_vars(nodes: Query<(Entity, &dyn GetVar, &GlobalTransform)>, mut commands: Commands) {
        for (e, gv, t) in &nodes {
            let mut vars: HashMap<VarName, VarValue> = default();
            let mut source: HashMap<VarName, NodeKind> = default();
            vars.insert(VarName::position, t.translation().xy().into());
            for v in gv {
                let kind = v.kind();
                for (var, value) in v.get_all_vars() {
                    source.insert(var, kind);
                    vars.insert(var, value);
                }
            }
            commands.entity(e).insert(NodeState { vars, source });
        }
    }
    pub fn collect_full_state(
        In(entity): In<Entity>,
        nodes: Query<(&dyn GetVar, Option<&Parent>)>,
    ) -> NodeState {
        dbg!("collect full state");
        let mut state = NodeState::default();
        let mut entity = Some(entity);
        for (n, _) in nodes.iter() {
            for n in n {
                dbg!(n.kind());
            }
        }
        while let Some((gv, p)) = entity.and_then(|e| nodes.get(e).ok()) {
            for v in gv {
                for (var, value) in v.get_all_vars() {
                    if !state.contains(var) {
                        state.insert(v.kind(), var, value);
                    }
                }
            }
            entity = p.map(|p| p.get());
        }
        state
    }
}
