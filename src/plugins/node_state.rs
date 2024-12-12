use bevy::{app::PreUpdate, math::Vec3Swizzles, prelude::Commands};

use super::*;

pub struct NodeStatePlugin;

impl Plugin for NodeStatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, Self::collect_vars);
    }
}

impl NodeStatePlugin {
    fn collect_vars(
        mut nodes: Query<(Entity, &dyn GetVar, &GlobalTransform)>,
        mut commands: Commands,
    ) {
        for (e, gv, t) in &mut nodes {
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
}
