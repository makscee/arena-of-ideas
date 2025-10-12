use super::*;
use crate::ui::Confirmation;
use spacetimedb_sdk::DbContext;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

mod relationships;
pub use relationships::*;

pub struct ExplorerPlugin;

impl Plugin for ExplorerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ExplorerState>()
            .add_systems(OnEnter(GameState::Explorer), Self::init_state);
    }
}

#[derive(Resource, Default)]
pub struct ExplorerState {
    // Store inspected node id for each named node kind
    inspected: HashMap<NamedNodeKind, u64>,
    // Store current and selected node id for each node kind
    current: HashMap<NodeKind, u64>,
    selected: HashMap<NodeKind, u64>,
    // Which node is actively being edited/viewed
    active_node: Option<(NodeKind, u64)>,
    // Cache of named nodes for UI lists
    named_nodes: HashMap<NamedNodeKind, Vec<(u64, String, i32)>>,
    // View mode for content panes
    view_mode: HashMap<NodeKind, ViewMode>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ViewMode {
    Current,
    Selected,
}

#[derive(PartialEq, Eq, Clone, Copy, Hash, AsRefStr, Serialize, Deserialize, Debug, Display)]
pub enum ExplorerPane {
    NamedList(NamedNodeKind),
    NamedCard(NamedNodeKind),
    ContentPane(NodeKind),
    NamedSelector(NamedNodeKind),
}

impl ExplorerPlugin {
    fn init_state(mut state: ResMut<ExplorerState>) {
        state.refresh_named_cache();
    }

    pub fn pane(pane: ExplorerPane, ui: &mut Ui, world: &mut World) {
        match pane {
            ExplorerPane::NamedList(kind) => match kind {
                NamedNodeKind::NHouse => Self::render_named_list::<NHouse>(ui, world),
                NamedNodeKind::NUnit => Self::render_named_list::<NUnit>(ui, world),
                NamedNodeKind::NAbilityMagic => Self::render_named_list::<NAbilityMagic>(ui, world),
                NamedNodeKind::NStatusMagic => Self::render_named_list::<NStatusMagic>(ui, world),
            },
            ExplorerPane::NamedCard(kind) => match kind {
                NamedNodeKind::NHouse => Self::render_named_card::<NHouse>(ui, world),
                NamedNodeKind::NUnit => Self::render_named_card::<NUnit>(ui, world),
                NamedNodeKind::NAbilityMagic => Self::render_named_card::<NAbilityMagic>(ui, world),
                NamedNodeKind::NStatusMagic => Self::render_named_card::<NStatusMagic>(ui, world),
            },
            ExplorerPane::ContentPane(kind) => match kind {
                NodeKind::NHouseColor => Self::render_content_pane::<NHouseColor>(ui, world),
                NodeKind::NUnitDescription => {
                    Self::render_content_pane::<NUnitDescription>(ui, world)
                }
                NodeKind::NUnitBehavior => Self::render_content_pane::<NUnitBehavior>(ui, world),
                NodeKind::NUnitRepresentation => {
                    Self::render_content_pane::<NUnitRepresentation>(ui, world)
                }
                NodeKind::NUnitStats => Self::render_content_pane::<NUnitStats>(ui, world),
                NodeKind::NStatusDescription => {
                    Self::render_content_pane::<NStatusDescription>(ui, world)
                }
                NodeKind::NStatusBehavior => {
                    Self::render_content_pane::<NStatusBehavior>(ui, world)
                }
                NodeKind::NStatusRepresentation => {
                    Self::render_content_pane::<NStatusRepresentation>(ui, world)
                }
                NodeKind::NAbilityDescription => {
                    Self::render_content_pane::<NAbilityDescription>(ui, world)
                }
                NodeKind::NAbilityEffect => Self::render_content_pane::<NAbilityEffect>(ui, world),
                _ => {}
            },
            ExplorerPane::NamedSelector(kind) => Self::render_named_selector(kind, ui, world),
        }
    }

    fn render_named_list<T: ClientNode + NamedNode + FTitle + bevy::ecs::component::Component>(
        ui: &mut Ui,
        world: &mut World,
    ) {
        let kind: NamedNodeKind = T::kind_s().try_into().unwrap();

        let state = world.remove_resource::<ExplorerState>().unwrap();

        let items = state.named_nodes.get(&kind).cloned();
        let inspected_id = state.inspected.get(&kind).copied();

        if let Some(items) = items {
            world
                .with_context(|context| {
                    items
                        .as_list(|item, ctx, ui| {
                            let item_id = item.0;
                            let is_inspected = inspected_id == Some(item_id);

                            ui.set_width(ui.available_width());

                            if is_inspected {
                                ui.visuals_mut().override_text_color = Some(ctx.color(ui));
                            }

                            ui.horizontal(|ui| {
                                ui.label(format!("[{}]", item.2));
                                if let Ok(node) = ctx.load::<T>(item_id) {
                                    node.as_title().compose(ctx, ui);
                                }
                            })
                            .response
                        })
                        .with_hover(|item, _, ui| {
                            let id = item.0;
                            if ui.button("Inspect").clicked() {
                                op(move |world| {
                                    let mut state = world.resource_mut::<ExplorerState>();
                                    state.set_inspected(kind, id);
                                });
                            }
                        })
                        .compose(context, ui);
                    Ok(())
                })
                .ui(ui);
        } else {
            ui.centered_and_justified(|ui| {
                ui.vertical(|ui| {
                    ui.label(format!("No {} available", kind));
                    ui.add_space(10.0);
                    if ui.button(format!("➕ Create New {}", kind)).clicked() {
                        match kind {
                            NamedNodeKind::NHouse => Self::open_content_creator::<NHouse>(world),
                            NamedNodeKind::NUnit => Self::open_content_creator::<NUnit>(world),
                            NamedNodeKind::NAbilityMagic => {
                                Self::open_content_creator::<NAbilityMagic>(world)
                            }
                            NamedNodeKind::NStatusMagic => {
                                Self::open_content_creator::<NStatusMagic>(world)
                            }
                        }
                    }
                });
            });
        }

        world.insert_resource(state);
    }

    fn render_named_card<T: ClientNode + NamedNode + FCard>(ui: &mut Ui, world: &mut World) {
        let kind: NamedNodeKind = T::kind_s().try_into().unwrap();
        let state = world.resource::<ExplorerState>();

        if let Some(&node_id) = state.inspected.get(&kind) {
            world
                .with_context(|ctx| {
                    if let Ok(entity) = ctx.entity(node_id) {
                        if let Ok(node) = ctx.load_entity::<T>(entity) {
                            let size = ui.available_size();
                            node.render_card(ui, size);
                        }
                    }
                    Ok(())
                })
                .ui(ui);
        } else {
            ui.centered_and_justified(|ui| {
                ui.vertical(|ui| {
                    ui.label(format!("Select a {} to preview", kind));
                    ui.add_space(10.0);
                    if ui.button(format!("➕ Create New {}", kind)).clicked() {
                        match kind {
                            NamedNodeKind::NHouse => Self::open_content_creator::<NHouse>(world),
                            NamedNodeKind::NUnit => Self::open_content_creator::<NUnit>(world),
                            NamedNodeKind::NAbilityMagic => {
                                Self::open_content_creator::<NAbilityMagic>(world)
                            }
                            NamedNodeKind::NStatusMagic => {
                                Self::open_content_creator::<NStatusMagic>(world)
                            }
                        }
                    }
                });
            });
        }
    }

    fn render_content_pane<T: ClientNode + FDisplay + FEdit + FTitle + Serialize>(
        ui: &mut Ui,
        world: &mut World,
    ) {
        let kind = T::kind_s();
        let mut state = world.remove_resource::<ExplorerState>().unwrap();
        let view_mode = *state.view_mode.entry(kind).or_insert(ViewMode::Current);

        let parent_kind = get_named_parent(kind);
        let has_inspected_parent = parent_kind
            .and_then(|pk| state.inspected.get(&pk).copied())
            .is_some();

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                if ui
                    .selectable_label(view_mode == ViewMode::Current, "Current")
                    .clicked()
                {
                    state.view_mode.insert(kind, ViewMode::Current);
                }
                if ui
                    .selectable_label(view_mode == ViewMode::Selected, "Selected")
                    .clicked()
                {
                    state.view_mode.insert(kind, ViewMode::Selected);
                }
            });

            ui.separator();

            let node_id = if view_mode == ViewMode::Selected {
                state.selected.get(&kind).copied()
            } else {
                state.current.get(&kind).copied()
            };

            ui.horizontal_wrapped(|ui| {
                if has_inspected_parent && node_id.is_some() {
                    if ui.button("🔄 Select Different").clicked() {
                        Self::open_content_selector::<T>(world, &state);
                    }
                    ui.add_space(5.0);
                }
                if has_inspected_parent && ui.button("➕ Add New").clicked() {
                    Self::open_content_creator::<T>(world);
                }
            });

            if let Some(node_id) = node_id {
                world
                    .with_context(|context| {
                        context.load::<T>(node_id)?.display(context, ui);
                        Ok(())
                    })
                    .ui(ui);
            } else if has_inspected_parent {
                ui.centered_and_justified(|ui| {
                    ui.label("No content available - click 'Add New' to create");
                });
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Select a parent node to view content");
                });
            }
        });
        world.insert_resource(state);
    }

    fn render_named_selector(selector_kind: NamedNodeKind, ui: &mut Ui, world: &mut World) {
        let state = world.resource::<ExplorerState>();

        let nodes_to_show = state.named_nodes.get(&selector_kind);
        let current_parent = match selector_kind {
            NamedNodeKind::NUnit => {
                // For units, parent could be either ability or status
                state
                    .inspected
                    .get(&NamedNodeKind::NAbilityMagic)
                    .copied()
                    .or_else(|| state.inspected.get(&NamedNodeKind::NStatusMagic).copied())
            }
            NamedNodeKind::NAbilityMagic | NamedNodeKind::NStatusMagic => {
                state.inspected.get(&NamedNodeKind::NHouse).copied()
            }
            _ => None,
        };

        ui.vertical(|ui| {
            ui.heading(format!("Select {}", selector_kind));

            if let Some(nodes) = nodes_to_show {
                for (id, name, _rating) in nodes {
                    ui.horizontal(|ui| {
                        let is_linked = if let Some(parent_id) = current_parent {
                            cn().db()
                                .node_links()
                                .iter()
                                .any(|link| link.parent == parent_id && link.child == *id)
                        } else {
                            false
                        };

                        if selector_kind == NamedNodeKind::NUnit {
                            let mut checked = is_linked;
                            if ui.checkbox(&mut checked, "").changed() {
                                if checked {
                                    if let Some(parent_id) = current_parent {
                                        cn().reducers
                                            .content_select_link(parent_id, *id)
                                            .notify_error_op();
                                    }
                                }
                            }
                        }

                        let id = *id;
                        if ui.link(name).clicked() {
                            op(move |world| {
                                let mut state = world.resource_mut::<ExplorerState>();
                                state.set_inspected(selector_kind, id);
                            });
                        }
                    });
                }
            }
        });
    }

    fn open_content_selector<T: ClientNode + FTitle>(world: &mut World, state: &ExplorerState) {
        let kind = T::kind_s();

        let parent_kind = get_named_parent(kind);
        let parent_id = parent_kind.and_then(|pk| state.inspected.get(&pk).copied());

        if parent_id.is_none() {
            return;
        }
        let parent_id = parent_id.unwrap();

        let nodes: Vec<T> = cn()
            .db()
            .nodes_world()
            .iter()
            .filter(|n| n.kind == kind.as_ref() && (n.owner == ID_CORE || n.owner == 0))
            .filter_map(|n| n.to_node().ok())
            .collect();

        if nodes.is_empty() {
            return;
        }

        let current_selected = state.current.get(&kind).copied();
        let user_selected = state.selected.get(&kind).copied();

        Confirmation::new("Select Content")
            .content(move |ui, world| {
                let mut selected = None;
                world
                    .with_context(|ctx| {
                        for node in &nodes {
                            let node_id = node.id();
                            ui.horizontal(|ui| {
                                node.as_title().compose(ctx, ui);

                                if Some(node_id) == current_selected {
                                    ui.label("●");
                                }
                                if Some(node_id) == user_selected {
                                    ui.label("★");
                                }

                                if ui.button("Select").clicked() {
                                    selected = Some(node_id);
                                }
                            });
                        }
                        Ok(())
                    })
                    .ui(ui);

                if let Some(id) = selected {
                    cn().reducers
                        .content_select_link(parent_id, id)
                        .notify_error(world);

                    world
                        .resource_mut::<ExplorerState>()
                        .selected
                        .insert(kind, id);

                    return true;
                }
                false
            })
            .push(world);
    }

    fn open_content_creator<T: ClientNode + Default + FEdit + Clone + Serialize + 'static>(
        world: &mut World,
    ) {
        let kind = T::kind_s();

        let new_node = Arc::new(Mutex::new(T::default()));
        let new_node_clone = new_node.clone();

        Confirmation::new(&format!("Create New {}", kind))
            .accept_name("Publish")
            .cancel_name("Cancel")
            .content(move |ui, world| {
                ui.vertical(|ui| {
                    ui.label("Edit new node:");
                    ui.separator();

                    world
                        .with_context(|ctx| {
                            if let Ok(mut node) = new_node.lock() {
                                node.edit(ctx, ui);
                            }
                            Ok(())
                        })
                        .ui(ui);
                });
                false
            })
            .accept(move |_| {
                if let Ok(node) = new_node_clone.lock() {
                    let node = node.clone();
                    Self::publish_new_node::<T>(node);
                }
            })
            .push(world);
    }

    fn publish_new_node<T: ClientNode + Clone + Serialize>(node: T) {
        let packed_string = ron::to_string(&node.pack()).unwrap_or_default();
        cn().reducers
            .content_publish_node(packed_string)
            .notify_error_op();
        op(|world| {
            let mut state = world.resource_mut::<ExplorerState>();
            state.refresh_named_cache();
        });
    }
}

impl ExplorerState {
    pub fn set_inspected(&mut self, kind: NamedNodeKind, node_id: u64) {
        self.inspected.insert(kind, node_id);

        // Update linked nodes based on relationships
        self.update_linked_chain(kind, node_id);
    }

    fn update_linked_chain(&mut self, inspected_kind: NamedNodeKind, inspected_id: u64) {
        // Clear previous chain
        self.current.clear();
        self.selected.clear();

        // Get related node kinds for this inspected type
        let related = get_related_nodes(inspected_kind);

        // For each related node, find and set the linked node
        for node_kind in related {
            if let Some(linked_id) = self.find_linked_node(inspected_id, node_kind) {
                self.current.insert(node_kind, linked_id);
                // Initially, selected mirrors current
                self.selected.insert(node_kind, linked_id);
            }
        }
    }

    fn find_linked_node(&self, base_id: u64, target_kind: NodeKind) -> Option<u64> {
        let target_kind = target_kind.as_ref();
        let mut links: Vec<_> = default();
        for link in cn().db().node_links().iter() {
            if link.parent == base_id && link.child_kind == target_kind {
                links.push((link.child, link.rating));
            } else if link.child == base_id && link.parent_kind == target_kind {
                links.push((link.parent, link.rating));
            }
        }
        links.sort_by_key(|&(_, rating)| rating);
        links.last().map(|&(id, _)| id)
    }

    pub fn refresh_named_cache(&mut self) {
        self.named_nodes.clear();

        for kind in NamedNodeKind::iter() {
            let kind_str = kind.as_ref();

            let mut nodes: Vec<(u64, String, i32)> = cn()
                .db()
                .nodes_world()
                .iter()
                .filter(|n| n.kind == kind_str && (n.owner == ID_CORE || n.owner == 0))
                .map(|n| {
                    let name = if !n.data.is_empty() {
                        todo!()
                    } else {
                        format!("{}#{}", kind, n.id)
                    };
                    (n.id, name, n.rating)
                })
                .collect();

            nodes.sort_by(|a, b| b.2.cmp(&a.2));
            self.named_nodes.insert(kind, nodes);
        }
    }

    pub fn get_active_node(&self) -> Option<(NodeKind, u64)> {
        self.active_node
    }

    pub fn set_active_node(&mut self, kind: NodeKind, id: u64) {
        self.active_node = Some((kind, id));
    }
}
