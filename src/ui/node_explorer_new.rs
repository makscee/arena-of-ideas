use super::*;

pub struct NodeExplorerPluginNew;

impl Plugin for NodeExplorerPluginNew {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Explorer), Self::load_kinds);
    }
}

#[derive(Resource)]
struct NodeExplorerData {
    nodes: HashMap<NodeKind, Vec<u64>>,
}

impl NodeExplorerPluginNew {
    pub fn load_kinds(world: &mut World) {
        let mut nodes: HashMap<NodeKind, Vec<u64>> = HashMap::new();
        for node in cn().db.nodes_world().iter() {
            if node.owner != ID_CORE {
                continue;
            }
            nodes
                .entry(node.kind())
                .or_insert_with(Vec::new)
                .push(node.id);
        }
        world.insert_resource(NodeExplorerData { nodes });
    }
    pub fn pane_kind_list(
        ui: &mut Ui,
        world: &mut World,
        kind: NodeKind,
    ) -> Result<(), ExpressionError> {
        Context::from_world_r(world, |context| {
            let data = context.world()?.resource::<NodeExplorerData>();
            let Some(nodes) = data.nodes.get(&kind) else {
                return Err(
                    ExpressionErrorVariants::NotFound(format!("No nodes of kind {kind}")).into(),
                );
            };
            kind.show_explorer(context, ViewContext::new(ui), ui, nodes, None)?;
            Ok(())
        })
    }
}
