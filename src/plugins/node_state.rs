use bevy::{app::PreUpdate, math::Vec3Swizzles, prelude::Commands};

use super::*;

pub struct NodeStatePlugin;

impl Plugin for NodeStatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, Self::collect_vars)
            .register_type::<NodeState>();
    }
}

impl NodeStatePlugin {
    fn collect_vars(
        mut nodes: Query<(Entity, &dyn GetVar, &GlobalTransform)>,
        mut commands: Commands,
    ) {
        for (e, gv, t) in &mut nodes {
            let mut vars: HashMap<VarName, VarValue> = default();
            vars.insert(VarName::position, t.translation().xy().into());
            for v in gv {
                vars.extend(v.get_all_vars());
            }
            commands.entity(e).insert(NodeState { vars });
        }
    }
}
