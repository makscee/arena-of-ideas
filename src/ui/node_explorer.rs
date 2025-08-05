use std::{marker::PhantomData, ops::Not};

use bevy_egui::egui::Grid;

use super::*;

#[derive(Resource, Default, Clone)]
pub struct NodeExplorerData {
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

pub struct NodesListWidget<T: NodeViewFns> {
    pd: PhantomData<T>,
}

impl<T: NodeViewFns> NodesListWidget<T> {
    pub fn new() -> NodesListWidget<T> {
        Self { pd: PhantomData }
    }
    pub fn ui(
        &mut self,
        context: &Context,
        vctx: ViewContext,
        ui: &mut Ui,
        ids: &Vec<u64>,
        selected: Option<u64>,
    ) -> Result<Option<u64>, ExpressionError> {
        let mut new_selected: Option<u64> = None;
        ui.push_id(vctx.id, |ui| {
            let mut nodes = ids
                .into_iter()
                .filter_map(|id| context.get_by_id::<T>(*id).ok())
                .collect_vec();

            // Sort by rating (descending) then by node_id (ascending)
            nodes.sort_by(|a, b| {
                let rating_a = a.id().node_rating().unwrap_or_default();
                let rating_b = b.id().node_rating().unwrap_or_default();
                match rating_b.cmp(&rating_a) {
                    // descending rating
                    std::cmp::Ordering::Equal => a.id().cmp(&b.id()), // ascending id
                    other => other,
                }
            });
            let mut table = nodes.table().column(
                "r",
                move |context, ui, node, _value| {
                    node.see(context).node_rating(ui);
                    Ok(())
                },
                |_, node| Ok(VarValue::i32(node.id().node_rating().unwrap_or_default())),
            );
            if let Some((is_parent, id)) = vctx.link_rating {
                table = table.column(
                    "lr",
                    move |context, ui, node, _value| {
                        node.see(context).node_link_rating(ui, is_parent, id);
                        Ok(())
                    },
                    move |context, node| {
                        Ok(VarValue::i32(
                            node.get_node_link_rating(context, is_parent, id)
                                .unwrap_or_default()
                                .0,
                        ))
                    },
                );
            }
            table = table
                .column(
                    "node",
                    |context, ui, node, _value| {
                        ui.horizontal(|ui| {
                            let response = node.view_title(
                                vctx.selected(selected.is_some_and(|id| id == node.id())),
                                context,
                                ui,
                            );
                            if node.owner() == ID_CORE {
                                ui.painter().rect_stroke(
                                    response.rect,
                                    ROUNDING,
                                    YELLOW.stroke(),
                                    egui::StrokeKind::Outside,
                                );
                            }
                            response.bar_menu(|ui| {
                                if "open in inspector".cstr().button(ui).clicked() {
                                    new_selected = Some(node.id());
                                    ui.close_menu();
                                }
                            });
                            node.view_data(vctx.one_line(true), context, ui);
                        });
                        Ok(())
                    },
                    |_, node| Ok(VarValue::String(node.get_data())),
                )
                .column_remainder();
            table.ui(context, ui);
        });
        Ok(new_selected)
    }
}

pub struct NodeExplorerPlugin;

impl Plugin for NodeExplorerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Explorer), Self::load_kinds)
            .add_systems(OnEnter(GameState::Inspector), Self::init_inspector);
    }
}

impl NodeExplorerPlugin {
    pub fn load_kinds(world: &mut World) {
        let mut data = NodeExplorerData::default();
        Self::load_kinds_internal(&mut data);

        let kind = NodeKind::NHouse;
        data.selected_kind = kind;
        data.selected_ids = kind.query_all_ids(world);
        data.owner_filter = OwnerFilter::Content; // Default to Content filter (0 + ID_CORE)

        world.insert_resource(data);
    }

    pub fn init_inspector(world: &mut World) {
        if let Some(mut existing_data) = world.remove_resource::<NodeExplorerData>() {
            if existing_data.inspected_node.is_some() {
                // We have a selected node, ensure all data is properly loaded
                Self::load_kinds_internal(&mut existing_data);
                world.insert_resource(existing_data);
                return;
            }
            world.insert_resource(existing_data);
        }
        let mut data = NodeExplorerData::default();
        Self::load_kinds_internal(&mut data);

        let kind = NodeKind::NHouse;
        data.selected_kind = kind;
        data.selected_ids = kind.query_all_ids(world);
        data.owner_filter = OwnerFilter::Content;
        world.insert_resource(data);
    }

    fn load_kinds_internal(data: &mut NodeExplorerData) {
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
        data: &mut NodeExplorerData,
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
        for child in kind.all_linked_children() {
            if !data.children.keys().contains(&child) {
                data.children.insert(child, default());
            }
        }
        for parent in kind.all_linked_parents() {
            if !data.parents.keys().contains(&parent) {
                data.parents.insert(parent, default());
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
            .get_resource::<NodeExplorerData>()
            .and_then(|data| data.inspected_node)
    }

    pub fn set_inspected_node(world: &mut World, node_id: u64) -> Result<(), ExpressionError> {
        let mut data = world
            .remove_resource::<NodeExplorerData>()
            .unwrap_or_default();

        Context::from_world_r(world, |context| {
            Self::select_node(context, &mut data, node_id)?;
            Ok(())
        })?;

        world.insert_resource(data);
        Ok(())
    }

    pub fn has_inspector_data(world: &World) -> bool {
        if let Some(data) = world.get_resource::<NodeExplorerData>() {
            data.inspected_node.is_some()
        } else {
            false
        }
    }

    pub fn pane_selected(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        let mut ned = world
            .remove_resource::<NodeExplorerData>()
            .to_e_not_found()?;
        let r = Context::from_world_r(world, |context| {
            let kind = ned.selected_kind;
            if let Some(selected) = kind.show_explorer(
                context,
                ViewContext::new(ui).one_line(true),
                ui,
                &ned.selected_ids,
                ned.inspected_node,
            )? {
                Self::select_node(context, &mut ned, selected)?;
            }
            Ok(())
        });
        world.insert_resource(ned);
        r
    }
    pub fn pane_node(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        let ned = world.get_resource::<NodeExplorerData>().to_e_not_found()?;
        let Some(id) = ned.inspected_node else {
            return Ok(());
        };
        let node = id.get_node().to_e_not_found()?;
        node.kind().cstr().label(ui);
        Context::from_world_r(world, |context| {
            format!(
                "[tw #]{id} [tw e:]{} [tw owner:] {}",
                context.entity(id)?,
                node.owner
            )
            .cstr()
            .label(ui);

            if format!("[red delete] #{id}").cstr().button(ui).clicked() {
                cn().reducers.admin_delete_node(id).unwrap();
            }
            let ns = context.get_by_id::<NodeState>(id)?;
            match ns.kind {
                NodeKind::NHouse => context
                    .get_by_id::<NHouse>(id)?
                    .see(context)
                    .tag_card(ui)
                    .ui(ui),
                NodeKind::NActionAbility => context
                    .get_by_id::<NActionAbility>(id)?
                    .see(context)
                    .tag_card(ui)
                    .ui(ui),
                NodeKind::NStatusAbility => context
                    .get_by_id::<NStatusAbility>(id)?
                    .see(context)
                    .tag_card(ui)
                    .ui(ui),
                NodeKind::NFusion => context
                    .get_by_id::<NFusion>(id)?
                    .show_card(context, ui)
                    .ui(ui),
                NodeKind::NUnit => context
                    .get_by_id::<NUnit>(id)?
                    .see(context)
                    .tag_card(ui)
                    .ui(ui),
                NodeKind::NUnitRepresentation => {
                    context.get_by_id::<NUnitRepresentation>(id)?.view(
                        ViewContext::new(ui),
                        context,
                        ui,
                    );
                }
                _ => {}
            }
            Grid::new("vars").show(ui, |ui| {
                for (var, state) in &ns.vars {
                    var.cstr().label(ui);
                    state.value.cstr().label(ui);
                    ui.end_row();
                }
            });
            Ok(())
        })
    }
    pub fn pane_linked_nodes(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        let mut ned = world
            .remove_resource::<NodeExplorerData>()
            .to_e_not_found()?;
        Context::from_world_r(world, |context| {
            let mut selected: Option<u64> = None;
            let inspected_id = ned.inspected_node.unwrap_or(0);

            // Collect all nodes by kind with their relationship type and link rating
            let mut nodes_by_kind: std::collections::HashMap<NodeKind, Vec<(u64, String, i32)>> =
                std::collections::HashMap::new();

            // Add parents
            for (kind, ids) in &ned.parents {
                for &id in ids {
                    let rating = context
                        .world()?
                        .get_any_link_rating(id, inspected_id)
                        .map(|(r, _)| r)
                        .unwrap_or(0);
                    nodes_by_kind.entry(*kind).or_default().push((
                        id,
                        "Parent".to_string(),
                        rating,
                    ));
                }
            }

            // Add children
            for (kind, ids) in &ned.children {
                for &id in ids {
                    let rating = context
                        .world()?
                        .get_any_link_rating(inspected_id, id)
                        .map(|(r, _)| r)
                        .unwrap_or(0);
                    nodes_by_kind
                        .entry(*kind)
                        .or_default()
                        .push((id, "Child".to_string(), rating));
                }
            }

            // Add unlinked nodes only for kinds that have linked nodes
            let linked_kinds: std::collections::HashSet<NodeKind> = ned
                .parents
                .keys()
                .chain(ned.children.keys())
                .copied()
                .collect();

            for kind in linked_kinds {
                let all_ids = kind.query_all_ids(context.world_mut()?);
                for id in all_ids {
                    if id != inspected_id
                        && !ned
                            .parents
                            .get(&kind)
                            .map_or(false, |ids| ids.contains(&id))
                        && !ned
                            .children
                            .get(&kind)
                            .map_or(false, |ids| ids.contains(&id))
                    {
                        nodes_by_kind.entry(kind).or_default().push((
                            id,
                            "Unlinked".to_string(),
                            0,
                        ));
                    }
                }
            }

            // Sort each kind's nodes by rating (descending), then by relationship type
            for (_, nodes) in nodes_by_kind.iter_mut() {
                nodes.sort_by(|a, b| b.2.cmp(&a.2).then_with(|| a.1.cmp(&b.1)));
            }

            let mut kinds: Vec<_> = nodes_by_kind.keys().copied().collect();
            kinds.sort();

            if kinds.is_empty() {
                ui.label("No nodes found");
                return Ok(());
            }

            ui.columns(kinds.len(), |columns| {
                for (i, kind) in kinds.iter().enumerate() {
                    if let Some(nodes) = nodes_by_kind.get(kind) {
                        columns[i].vertical(|ui| {
                            kind.cstr_c(ui.visuals().weak_text_color()).label(ui);

                            // Add new node creation for this specific kind
                            let state = ned.new_node_states.entry(*kind).or_default();
                            let mut is_open = state.is_open;
                            let mut pack = state.pack.take();

                            ScrollArea::vertical().id_salt(kind).show(ui, |ui| {
                                ui.collapsing(format!("New {}", kind.cstr()), |ui| {
                                    is_open = true;

                                    // Initialize pack if needed
                                    if pack.is_none() {
                                        let mut new_pack = PackedNodes::default();
                                        new_pack.root = 1;
                                        new_pack.add_node(kind.to_string(), kind.default_data(), 1);
                                        pack = Some(new_pack);
                                    }

                                    if let Some(ref mut pack) = pack {
                                        if let Ok(_view_response) =
                                            kind.view_pack_with_children_mut(context, ui, pack)
                                        {
                                            ui.horizontal(|ui| {
                                                if ui.button("Publish").clicked() {
                                                    let pack_string = to_ron_string(pack);
                                                    cn().reducers
                                                        .content_publish_node(pack_string)
                                                        .ok();
                                                    // Reset the pack after publishing
                                                    let mut new_pack = PackedNodes::default();
                                                    new_pack.root = 1;
                                                    new_pack.add_node(
                                                        kind.to_string(),
                                                        kind.default_data(),
                                                        1,
                                                    );
                                                    *pack = new_pack;
                                                }

                                                if ui.button("Reset").clicked() {
                                                    let mut new_pack = PackedNodes::default();
                                                    new_pack.root = 1;
                                                    new_pack.add_node(
                                                        kind.to_string(),
                                                        kind.default_data(),
                                                        1,
                                                    );
                                                    *pack = new_pack;
                                                }
                                            });
                                        }
                                    }
                                });

                                // Update state
                                ned.new_node_states.get_mut(kind).unwrap().is_open = is_open;
                                ned.new_node_states.get_mut(kind).unwrap().pack = pack;

                                ui.separator();

                                let ids: Vec<u64> = nodes.iter().map(|(id, _, _)| *id).collect();
                                let vctx = ViewContext::new(ui)
                                    .one_line(true)
                                    .link_rating(true, inspected_id)
                                    .link_rating(false, inspected_id);

                                if let Ok(Some(id)) =
                                    kind.show_explorer(context, vctx, ui, &ids, ned.inspected_node)
                                {
                                    selected = Some(id);
                                }
                            });
                        });
                    }
                }
            });

            if let Some(selected) = selected {
                Self::select_node(context, &mut ned, selected)?;
                ui.ctx().data_mut(|w| w.remove_by_type::<NodeKind>());
            }
            Ok(())
        })?;
        world.insert_resource(ned);
        Ok(())
    }
}
