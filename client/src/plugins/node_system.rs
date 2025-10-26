use super::*;

/// Plugin to add node system resources
pub struct NodeSystemPlugin;

impl Plugin for NodeSystemPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SmartNodeMap::default())
            .insert_resource(NodeLinks::default());
    }
}
