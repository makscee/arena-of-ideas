use bevy::{app::PreUpdate, math::Vec3Swizzles, prelude::In};

use super::*;

pub struct NodeStatePlugin;

impl Plugin for NodeStatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, Self::inject_vars);
    }
}

impl NodeStatePlugin {
    fn inject_vars(mut nodes: Query<(&mut NodeState, &dyn GetVar, &GlobalTransform)>) {
        let t = gt().play_head();
        for (mut state, gv, transform) in &mut nodes {
            state.insert(
                t,
                0.1,
                VarName::position,
                transform.translation().xy().into(),
                NodeKind::None,
            );
            for v in gv {
                let kind = v.kind();
                for (var, value) in v.get_all_vars() {
                    state.insert(t, 0.0, var, value, kind);
                }
            }
        }
    }
    pub fn inject_entity_vars(
        In((entity, t)): In<(Entity, f32)>,
        mut nodes: Query<(&dyn GetVar, &mut NodeState)>,
    ) {
        if let Ok((gv, mut state)) = nodes.get_mut(entity) {
            for v in gv {
                let kind = v.kind();
                for (var, value) in v.get_all_vars() {
                    state.insert(t, 0.0, var, value, kind);
                }
            }
        }
    }
    pub fn collect_full_state(
        In(entity): In<Entity>,
        nodes: Query<(&dyn GetVar, Option<&Parent>)>,
    ) -> NodeState {
        let mut state = NodeState::default();
        let mut entity = Some(entity);
        while let Some((gv, p)) = entity.and_then(|e| nodes.get(e).ok()) {
            for v in gv {
                for (var, value) in v.get_all_vars() {
                    if !state.contains(var) {
                        state.insert(0.0, 0.0, var, value, v.kind());
                    }
                }
            }
            entity = p.map(|p| p.get());
        }
        state
    }
}
