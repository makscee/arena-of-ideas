use crate::prelude::*;
use crate::resources::context::{NodeEntityMap, NodeLinks};
use bevy::prelude::*;

/// Plugin to add node system resources
pub struct NodeSystemPlugin;

impl Plugin for NodeSystemPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(NodeEntityMap::default())
            .insert_resource(NodeLinks::default());
    }
}
