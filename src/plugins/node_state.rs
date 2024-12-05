use bevy::{app::PreUpdate, prelude::Commands};

use super::*;

pub struct NodeStatePlugin;

impl Plugin for NodeStatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, Self::collect_vars)
            .register_type::<NodeState>();
    }
}

impl NodeStatePlugin {
    fn collect_vars(mut nodes: Query<(Entity, &dyn GetVar)>, mut commands: Commands) {
        for (e, gv) in &mut nodes {
            let mut vars: HashMap<VarName, VarValue> = default();
            for v in gv {
                vars.extend(v.get_all_vars());
            }
            commands.entity(e).insert(NodeState { vars });
        }
    }
}
