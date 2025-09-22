use super::*;
use crate::ui::{Confirmation, Table};
use spacetimedb_sdk::DbContext;
use std::collections::HashMap;

pub struct ExplorerPlugin;

impl Plugin for ExplorerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ExplorerState>()
            .init_resource::<ExplorerSelection>()
            .add_systems(OnEnter(GameState::Explorer), Self::init_cache);
    }
}

#[derive(Resource, Default)]
pub struct ExplorerState {
    // Cache for each named node kind
    named_nodes: HashMap<NamedNodeKind, Vec<(u64, String, i32)>>,

    // Track whether viewing current or selected parts
    view_mode: HashMap<NodeKind, ViewMode>,

    // Cache for content nodes
    content_cache: HashMap<NodeKind, Vec<TNode>>,
}

#[derive(Resource, Default)]
pub struct ExplorerSelection {
    // Currently selected nodes for viewing
    selected: HashMap<NamedNodeKind, u64>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ViewMode {
    Current,
    Selected,
}

#[derive(PartialEq, Eq, Clone, Copy, Hash, AsRefStr, Serialize, Deserialize, Debug, Display)]
pub enum ExplorerPane {
    // Lists for named nodes
    NamedList(NamedNodeKind),

    // Card previews
    NamedCard(NamedNodeKind),

    // Content node panes
    ContentPane(NodeKind),

    // Named node selection panes
    NamedSelector(NamedNodeKind),
}

impl ExplorerPlugin {
    fn init_cache(mut state: ResMut<ExplorerState>) {
        state.refresh_all_caches();
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

    // Generic list renderer for named nodes
    fn render_named_list<T: Node + NamedNode + FTitle + Component>(ui: &mut Ui, world: &mut World) {
        let kind: NamedNodeKind = T::kind_s().try_into().unwrap();

        let state = world.remove_resource::<ExplorerState>().unwrap();
        let selection = world.resource::<ExplorerSelection>();

        let items = state.named_nodes.get(&kind).cloned();
        let selected = selection.selected.get(&kind).copied();

        if let Some(items) = items {
            Context::from_world(world, |context| {
                Table::from_data(&items)
                    .column(
                        "Name",
                        |_, ui, item, _| {
                            let item_id = item.0;
                            let is_selected = selected == Some(item_id);
                            if let Ok(entity) = context.entity(item_id) {
                                if let Ok(node) = context.component::<T>(entity) {
                                    if node.as_title().compose(context, ui).clicked() {
                                        op(move |world| {
                                            world
                                                .resource_mut::<ExplorerSelection>()
                                                .selected
                                                .insert(kind, item_id);
                                        });
                                    }
                                } else {
                                    ui.label(&item.1);
                                }
                            } else {
                                ui.label(&item.1);
                            }
                            Ok(())
                        },
                        |_, item| Ok(VarValue::String(item.1.clone())),
                    )
                    .column(
                        "Rating",
                        |_, ui, item, _| {
                            ui.horizontal(|ui| {
                                ui.label(item.2.to_string());
                                let id = item.0;
                                if ui.button("↑").clicked() {
                                    cn().reducers.content_vote_node(id, true).notify_error_op();
                                }
                                if ui.button("↓").clicked() {
                                    cn().reducers.content_vote_node(id, false).notify_error_op();
                                }
                            });
                            Ok(())
                        },
                        |_, item| Ok(VarValue::i32(item.2)),
                    )
                    .ui(context, ui);
            });
        } else {
            ui.centered_and_justified(|ui| {
                ui.label(format!("No {} available", kind));
            });
        }

        // Reinsert the state
        world.insert_resource(state);
    }

    // Generic card preview renderer using FCard trait
    fn render_named_card<T: Node + NamedNode + FCard + Component>(ui: &mut Ui, world: &mut World) {
        let kind: NamedNodeKind = T::kind_s().try_into().unwrap();
        let selection = world.resource::<ExplorerSelection>();

        if let Some(&node_id) = selection.selected.get(&kind) {
            Context::from_world(world, |context| {
                if let Ok(entity) = context.entity(node_id) {
                    if let Ok(node) = context.component::<T>(entity) {
                        let size = ui.available_size();
                        node.render_card(ui, size);
                    }
                }
            });
        } else {
            ui.centered_and_justified(|ui| {
                ui.label(format!("Select a {} to preview", kind));
            });
        }
    }

    // Generic content pane renderer
    fn render_content_pane<T: Node + FDisplay + FEdit + Component>(ui: &mut Ui, world: &mut World) {
        let kind = T::kind_s();
        let mut state = world.remove_resource::<ExplorerState>().unwrap();
        let view_mode = *state.view_mode.entry(kind).or_insert(ViewMode::Current);

        ui.vertical(|ui| {
            // View mode toggle
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

            // Get the currently selected named node based on context
            let selected_node_id = Self::get_current_selected_node(kind, world);

            if let Some(node_id) = selected_node_id {
                Context::from_world(world, |context| {
                    // Get linked content based on the parent node's type and field
                    // For example, if viewing NUnitDescription, we need to get it from the unit's description field
                    let linked_nodes = context.children(node_id);
                    let mut found = false;

                    for child_id in linked_nodes {
                        if let Ok(child_entity) = context.entity(child_id) {
                            if let Ok(content) = context.component::<T>(child_entity) {
                                content.display(context, ui);
                                found = true;
                                break;
                            }
                        }
                    }

                    if !found {
                        ui.label("No linked content");
                    }
                });
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Select a node to view its content");
                });
            }

            ui.separator();

            // Action buttons
            ui.horizontal(|ui| {
                if ui.button("Select Different").clicked() {
                    Self::open_content_selector::<T>(world, selected_node_id);
                }
                if ui.button("Create New").clicked() {
                    Self::open_content_creator::<T>(world, selected_node_id);
                }
            });
        });
        world.insert_resource(state);
    }

    // Generic named node selector
    fn render_named_selector(selector_kind: NamedNodeKind, ui: &mut Ui, world: &mut World) {
        let state = world.resource::<ExplorerState>();
        let selection = world.resource::<ExplorerSelection>();

        // Determine which nodes to show based on selector kind
        let (nodes_to_show, current_parent) = match selector_kind {
            NamedNodeKind::NHouse => {
                // For house selector, show all houses
                (state.named_nodes.get(&NamedNodeKind::NHouse), None)
            }
            NamedNodeKind::NUnit => {
                // For units selector (used in house view), show units belonging to selected house
                let selected_house = selection.selected.get(&NamedNodeKind::NHouse).copied();
                (state.named_nodes.get(&NamedNodeKind::NUnit), selected_house)
            }
            NamedNodeKind::NAbilityMagic => {
                let selected_house = selection.selected.get(&NamedNodeKind::NHouse).copied();
                (
                    state.named_nodes.get(&NamedNodeKind::NAbilityMagic),
                    selected_house,
                )
            }
            NamedNodeKind::NStatusMagic => {
                let selected_house = selection.selected.get(&NamedNodeKind::NHouse).copied();
                (
                    state.named_nodes.get(&NamedNodeKind::NStatusMagic),
                    selected_house,
                )
            }
        };

        ui.vertical(|ui| {
            ui.heading(format!("Select {}", selector_kind));

            if let Some(nodes) = nodes_to_show {
                for (id, name, _rating) in nodes {
                    ui.horizontal(|ui| {
                        // Check if this node is linked to parent
                        let is_linked = if let Some(parent_id) = current_parent {
                            // Check actual link in database
                            cn().db()
                                .node_links()
                                .iter()
                                .any(|link| link.parent == parent_id && link.child == *id)
                        } else {
                            false
                        };

                        if selector_kind == NamedNodeKind::NUnit {
                            // Multi-select for units
                            let mut checked = is_linked;
                            if ui.checkbox(&mut checked, "").changed() {
                                if checked {
                                    cn().reducers
                                        .content_select_link(current_parent.unwrap(), *id)
                                        .notify_error_op();
                                } else {
                                    // Unlink if needed
                                }
                            }
                        }

                        let id = *id;
                        if ui.link(name).clicked() {
                            // Switch to appropriate tab and select the node
                            op(move |world| {
                                world
                                    .resource_mut::<ExplorerSelection>()
                                    .selected
                                    .insert(selector_kind, id);
                            });
                        }
                    });
                }
            }
        });
    }

    // Helper functions
    fn get_current_selected_node(content_kind: NodeKind, world: &World) -> Option<u64> {
        let selection = world.resource::<ExplorerSelection>();

        // Determine which named node is currently selected based on content type
        match content_kind {
            NodeKind::NHouseColor => selection.selected.get(&NamedNodeKind::NHouse).copied(),
            NodeKind::NUnitDescription
            | NodeKind::NUnitBehavior
            | NodeKind::NUnitRepresentation
            | NodeKind::NUnitStats => selection.selected.get(&NamedNodeKind::NUnit).copied(),
            NodeKind::NStatusDescription
            | NodeKind::NStatusBehavior
            | NodeKind::NStatusRepresentation => selection
                .selected
                .get(&NamedNodeKind::NStatusMagic)
                .copied(),
            NodeKind::NAbilityDescription | NodeKind::NAbilityEffect => selection
                .selected
                .get(&NamedNodeKind::NAbilityMagic)
                .copied(),
            _ => None,
        }
    }

    fn open_content_selector<T: Node + Component>(world: &mut World, parent_id: Option<u64>) {
        if let Some(parent_id) = parent_id {
            let kind = T::kind_s();

            // Get available content nodes
            let nodes: Vec<u64> = cn()
                .db()
                .nodes_world()
                .iter()
                .filter(|n| n.kind == kind.as_ref() && (n.owner == ID_CORE || n.owner == 0))
                .map(|n| n.id)
                .collect();

            if !nodes.is_empty() {
                Confirmation::new("Select Content")
                    .content(move |ui, world| {
                        let mut selected = None;
                        for id in &nodes {
                            ui.horizontal(|ui| {
                                if ui.button(format!("Node #{}", id)).clicked() {
                                    selected = Some(*id);
                                }
                            });
                        }

                        // If something was selected, link it and close
                        if let Some(id) = selected {
                            cn().reducers
                                .content_select_link(parent_id, id)
                                .notify_error(world);
                            return true; // Close the confirmation
                        }
                        false
                    })
                    .push(world);
            }
        }
    }

    fn open_content_creator<T: Node + Component + Default>(
        world: &mut World,
        parent_id: Option<u64>,
    ) {
        if let Some(_parent_id) = parent_id {
            Confirmation::new("Create New Content")
                .content(move |ui, _world| {
                    ui.label(format!("Create new {}", T::kind_s()));
                    // TODO: Implement actual node creation with FEdit
                    false
                })
                .push(world);
        }
    }
}

impl ExplorerState {
    pub fn refresh_all_caches(&mut self) {
        // Refresh named nodes
        self.refresh_named_cache(NamedNodeKind::NHouse);
        self.refresh_named_cache(NamedNodeKind::NUnit);
        self.refresh_named_cache(NamedNodeKind::NAbilityMagic);
        self.refresh_named_cache(NamedNodeKind::NStatusMagic);

        // Refresh content nodes
        self.refresh_content_cache();
    }

    fn refresh_named_cache(&mut self, kind: NamedNodeKind) {
        let kind_str = match kind {
            NamedNodeKind::NHouse => NodeKind::NHouse.as_ref(),
            NamedNodeKind::NUnit => NodeKind::NUnit.as_ref(),
            NamedNodeKind::NAbilityMagic => NodeKind::NAbilityMagic.as_ref(),
            NamedNodeKind::NStatusMagic => NodeKind::NStatusMagic.as_ref(),
        };

        let mut nodes: Vec<(u64, String, i32)> = cn()
            .db()
            .nodes_world()
            .iter()
            .filter(|n| n.kind == kind_str && (n.owner == ID_CORE || n.owner == 0))
            .map(|n| {
                let name = if !n.data.is_empty() {
                    // Extract name from data based on node kind
                    match serde_json::from_str::<serde_json::Value>(&n.data) {
                        Ok(json) => json
                            .get(match kind {
                                NamedNodeKind::NHouse => "house_name",
                                NamedNodeKind::NUnit => "unit_name",
                                NamedNodeKind::NAbilityMagic => "ability_name",
                                NamedNodeKind::NStatusMagic => "status_name",
                            })
                            .and_then(|v| v.as_str())
                            .unwrap_or(&format!("{}#{}", kind, n.id))
                            .to_string(),
                        Err(_) => format!("{}#{}", kind, n.id),
                    }
                } else {
                    format!("{}#{}", kind, n.id)
                };
                (n.id, name, n.rating)
            })
            .collect();

        nodes.sort_by(|a, b| b.2.cmp(&a.2));
        self.named_nodes.insert(kind, nodes);
    }

    fn refresh_content_cache(&mut self) {
        self.content_cache.clear();

        let content_kinds = vec![
            NodeKind::NHouseColor,
            NodeKind::NUnitDescription,
            NodeKind::NUnitBehavior,
            NodeKind::NUnitRepresentation,
            NodeKind::NUnitStats,
            NodeKind::NStatusDescription,
            NodeKind::NStatusBehavior,
            NodeKind::NStatusRepresentation,
            NodeKind::NAbilityDescription,
            NodeKind::NAbilityEffect,
        ];

        for kind in content_kinds {
            let nodes: Vec<TNode> = cn()
                .db()
                .nodes_world()
                .iter()
                .filter(|n| n.kind == kind.as_ref() && (n.owner == ID_CORE || n.owner == 0))
                .map(|n| n.clone())
                .collect();
            self.content_cache.insert(kind, nodes);
        }
    }
}
