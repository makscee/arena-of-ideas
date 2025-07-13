use super::*;

pub struct NodeExplorerPluginNew;

impl Plugin for NodeExplorerPluginNew {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Explorer), Self::load_kinds);
    }
}

#[derive(Resource, Default)]
struct NodeExplorerData {
    nodes: HashMap<NodeKind, Vec<u64>>,
    new_node_states: HashMap<NodeKind, NewNodeState>,
}

#[derive(Default)]
struct NewNodeState {
    is_open: bool,
    pack: Option<PackedNodes>,
}

impl NodeExplorerPluginNew {
    pub fn load_kinds(world: &mut World) {
        let mut nodes: HashMap<NodeKind, Vec<u64>> = HashMap::new();
        for node in cn().db.nodes_world().iter() {
            if node.owner != ID_CORE && node.owner != 0 {
                continue;
            }
            nodes
                .entry(node.kind())
                .or_insert_with(Vec::new)
                .push(node.id);
        }
        // Sort each vector by rating (descending) then by node.id (ascending)
        for vec in nodes.values_mut() {
            vec.sort_by(|a, b| {
                let rating_a = a.node_rating().unwrap_or_default();
                let rating_b = b.node_rating().unwrap_or_default();
                match rating_b.cmp(&rating_a) {
                    // descending rating
                    std::cmp::Ordering::Equal => a.cmp(b), // ascending id
                    other => other,
                }
            });
        }
        world.insert_resource(NodeExplorerData {
            nodes,
            new_node_states: HashMap::new(),
        });
    }
    pub fn pane_kind_list(
        ui: &mut Ui,
        world: &mut World,
        kind: NodeKind,
    ) -> Result<(), ExpressionError> {
        let mut data = world
            .remove_resource::<NodeExplorerData>()
            .unwrap_or_default();

        let Some(nodes) = data.nodes.get(&kind).cloned() else {
            world.insert_resource(data);
            return Err(
                ExpressionErrorVariants::NotFound(format!("No nodes of kind {kind}")).into(),
            );
        };

        // Get or create new node state
        let state = data.new_node_states.entry(kind).or_default();
        let mut is_open = state.is_open;
        let mut pack = state.pack.take();

        let r = Context::from_world_r(world, |context| -> Result<(), ExpressionError> {
            // Show "New" collapsed header
            ui.collapsing("New", |ui| {
                is_open = true;

                // Initialize pack if needed
                if pack.is_none() {
                    let mut new_pack = PackedNodes::default();
                    new_pack.root = 1;
                    new_pack.add_node(kind.to_string(), kind.default_data(), 1);
                    pack = Some(new_pack);
                }

                if let Some(ref mut pack) = pack {
                    if let Ok(_view_response) = kind.view_pack_with_children_mut(context, ui, pack)
                    {
                        ui.horizontal(|ui| {
                            if ui.button("Publish").clicked() {
                                let pack_string = to_ron_string(pack);
                                cn().reducers.content_publish_node(pack_string).ok();
                                // Reset the pack after publishing
                                let mut new_pack = PackedNodes::default();
                                new_pack.root = 1;
                                new_pack.add_node(kind.to_string(), kind.default_data(), 1);
                                *pack = new_pack;
                            }

                            if ui.button("Reset").clicked() {
                                let mut new_pack = PackedNodes::default();
                                new_pack.root = 1;
                                new_pack.add_node(kind.to_string(), kind.default_data(), 1);
                                *pack = new_pack;
                            }
                        });
                    }
                }
            });

            // Show existing nodes
            kind.show_explorer(context, ViewContext::new(ui), ui, &nodes, None)?;
            Ok(())
        });

        // Update state
        data.new_node_states.get_mut(&kind).unwrap().is_open = is_open;
        data.new_node_states.get_mut(&kind).unwrap().pack = pack;

        world.insert_resource(data);
        r
    }
}
