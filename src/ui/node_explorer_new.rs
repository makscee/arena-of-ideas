use super::*;

pub struct NodeExplorerPluginNew;

impl Plugin for NodeExplorerPluginNew {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Explorer), Self::load_kinds)
            .add_systems(OnEnter(GameState::Inspector), Self::init_inspector);
    }
}

#[derive(Resource, Default, Clone)]
pub struct NodeExplorerDataNew {
    /// Nodes grouped by kind for list view
    pub nodes: HashMap<NodeKind, Vec<u64>>,
    /// State for creating new nodes of each kind
    pub new_node_states: HashMap<NodeKind, NewNodeState>,
    /// The node currently being inspected (maps to old 'selected')
    pub inspected_node: Option<u64>,
    /// The selected kind for filtering (from old NodeExplorerData)
    pub selected_kind: NodeKind,
    /// Selected IDs for the current kind (from old NodeExplorerData)
    pub selected_ids: Vec<u64>,
    /// Children of the inspected node, grouped by kind
    pub children: HashMap<NodeKind, Vec<u64>>,
    /// Parents of the inspected node, grouped by kind
    pub parents: HashMap<NodeKind, Vec<u64>>,
    /// Owner filter (from old NodeExplorerData)
    pub owner_filter: OwnerFilter,
}

#[derive(PartialEq, Eq, Clone, Copy, AsRefStr, Default, EnumIter)]
pub enum OwnerFilter {
    All,
    Core,
    #[default]
    Content,
}

impl ToCstr for OwnerFilter {
    fn cstr(&self) -> Cstr {
        self.as_ref().cstr()
    }
}

impl OwnerFilter {
    pub fn ids(self) -> HashSet<u64> {
        match self {
            OwnerFilter::All => default(),
            OwnerFilter::Core => [ID_CORE].into(),
            OwnerFilter::Content => [0, ID_CORE].into(),
        }
    }
}

#[derive(Default, Clone)]
pub struct NewNodeState {
    pub is_open: bool,
    pub pack: Option<PackedNodes>,
}

impl NodeExplorerPluginNew {
    pub fn load_kinds(world: &mut World) {
        let mut data = NodeExplorerDataNew::default();
        Self::load_kinds_internal(&mut data);

        let kind = NodeKind::NHouse;
        data.selected_kind = kind;
        data.selected_ids = kind.query_all_ids(world);
        data.owner_filter = OwnerFilter::Content; // Default to Content filter (0 + ID_CORE)

        world.insert_resource(data);
    }

    pub fn init_inspector(world: &mut World) {
        if let Some(mut existing_data) = world.remove_resource::<NodeExplorerDataNew>() {
            if existing_data.inspected_node.is_some() {
                // We have a selected node, ensure all data is properly loaded
                Self::load_kinds_internal(&mut existing_data);
                world.insert_resource(existing_data);
                return;
            }
            world.insert_resource(existing_data);
        }
        let mut data = NodeExplorerDataNew::default();
        Self::load_kinds_internal(&mut data);

        let kind = NodeKind::NHouse;
        data.selected_kind = kind;
        data.selected_ids = kind.query_all_ids(world);
        data.owner_filter = OwnerFilter::Content;
        world.insert_resource(data);
    }

    fn load_kinds_internal(data: &mut NodeExplorerDataNew) {
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
        data.nodes = nodes;
    }

    fn select_node(
        context: &mut Context,
        data: &mut NodeExplorerDataNew,
        id: u64,
    ) -> Result<(), ExpressionError> {
        data.inspected_node = Some(id);
        let kind = context.get_by_id::<NodeState>(id)?.kind;
        data.selected_kind = kind;

        let filter_ids = data.owner_filter.ids();
        data.selected_ids = kind
            .query_all_ids(context.world_mut()?)
            .into_iter()
            .filter(|id| {
                id.get_node()
                    .is_some_and(|node| filter_ids.is_empty() || filter_ids.contains(&node.owner))
            })
            .collect();

        data.children.clear();
        data.parents.clear();

        let filter_ids = data.owner_filter.ids();

        for child in context.children(id) {
            if let Some(child_node) = child.get_node() {
                if filter_ids.is_empty() || filter_ids.contains(&child_node.owner) {
                    let kind = child.kind()?;
                    data.children.entry(kind).or_default().push(child);
                }
            }
        }
        for parent in context.parents(id) {
            if let Some(parent_node) = parent.get_node() {
                if filter_ids.is_empty() || filter_ids.contains(&parent_node.owner) {
                    let kind = parent.kind()?;
                    data.parents.entry(kind).or_default().push(parent);
                }
            }
        }
        Ok(())
    }

    pub fn pane_kind_list(
        ui: &mut Ui,
        world: &mut World,
        kind: NodeKind,
    ) -> Result<(), ExpressionError> {
        let mut data = world
            .remove_resource::<NodeExplorerDataNew>()
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

        let mut should_switch_to_inspector = false;

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

            if let Some(selected) = kind.show_explorer(
                context,
                ViewContext::new(ui),
                ui,
                &nodes,
                data.inspected_node,
            )? {
                Self::select_node(context, &mut data, selected)?;
                should_switch_to_inspector = true;
            }
            Ok(())
        });

        // Update state
        data.new_node_states.get_mut(&kind).unwrap().is_open = is_open;
        data.new_node_states.get_mut(&kind).unwrap().pack = pack;

        world.insert_resource(data);

        if should_switch_to_inspector {
            GameState::Inspector.set_next(world);
        }

        r
    }

    pub fn get_inspected_node(world: &World) -> Option<u64> {
        world
            .get_resource::<NodeExplorerDataNew>()
            .and_then(|data| data.inspected_node)
    }

    pub fn set_inspected_node(world: &mut World, node_id: u64) -> Result<(), ExpressionError> {
        let mut data = world
            .remove_resource::<NodeExplorerDataNew>()
            .unwrap_or_default();

        Context::from_world_r(world, |context| {
            Self::select_node(context, &mut data, node_id)?;
            Ok(())
        })?;

        world.insert_resource(data);
        Ok(())
    }

    pub fn has_inspector_data(world: &World) -> bool {
        if let Some(data) = world.get_resource::<NodeExplorerDataNew>() {
            data.inspected_node.is_some()
        } else {
            false
        }
    }
}
