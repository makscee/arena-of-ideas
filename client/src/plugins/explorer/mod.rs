use super::*;
use crate::ui::Confirmation;
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
    named_nodes: HashMap<NamedNodeKind, Vec<(u64, String, i32)>>,
    view_mode: HashMap<NodeKind, ViewMode>,
    content_cache: HashMap<NodeKind, Vec<TNode>>,
    active_player_id: Option<u64>,
    player_selections_cache: HashMap<NodeKind, Option<u64>>,
}

#[derive(Resource, Default)]
pub struct ExplorerSelection {
    selected: HashMap<NamedNodeKind, u64>,
    content_selected: HashMap<(NodeKind, u64), u64>,
    content_current: HashMap<(NodeKind, u64), u64>,
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

    fn render_named_list<T: Node + NamedNode + FTitle + Component>(ui: &mut Ui, world: &mut World) {
        let kind: NamedNodeKind = T::kind_s().try_into().unwrap();

        let state = world.remove_resource::<ExplorerState>().unwrap();
        let selection = world.resource::<ExplorerSelection>();

        let items = state.named_nodes.get(&kind).cloned();
        let selected_id = selection.selected.get(&kind).copied();

        if let Some(items) = items {
            Context::from_world(world, |context| {
                items
                    .as_list(|item, context, ui| {
                        let item_id = item.0;
                        let is_selected = selected_id == Some(item_id);

                        ui.set_width(ui.available_width());

                        if is_selected {
                            ui.visuals_mut().override_text_color = Some(context.color(ui));
                        }

                        ui.horizontal(|ui| {
                            ui.label(format!("[{}]", item.2));
                            if let Ok(node) = context.component_by_id::<T>(item_id) {
                                node.as_title().compose(context, ui);
                            } else {
                            }
                        })
                        .response
                    })
                    .with_hover(|item, _, ui| {
                        let id = item.0;
                        if ui.button("Select").clicked() {
                            op(move |world| {
                                world
                                    .resource_mut::<ExplorerSelection>()
                                    .selected
                                    .insert(kind, id);
                            });
                        }
                    })
                    .compose(context, ui);
            });
        } else {
            ui.centered_and_justified(|ui| {
                ui.label(format!("No {} available", kind));
            });
        }

        world.insert_resource(state);
    }

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

    fn render_content_pane<T: Node + FDisplay + FEdit + FTitle + Component>(
        ui: &mut Ui,
        world: &mut World,
    ) {
        let kind = T::kind_s();
        let mut state = world.remove_resource::<ExplorerState>().unwrap();
        let view_mode = *state.view_mode.entry(kind).or_insert(ViewMode::Current);

        kind.cstr().label(ui);
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
                    state.cache_player_selection(kind);
                }
            });

            ui.separator();

            let selected_node_id = if view_mode == ViewMode::Selected {
                state.get_player_selected_node(kind)
            } else {
                Self::get_current_selected_node(kind, world)
            };

            if let Some(node_id) = selected_node_id {
                Context::from_world(world, |context| match context.top_linked::<T>(node_id) {
                    Ok(content) => {
                        content.display(context, ui);
                    }
                    Err(_) => {
                        ui.label("No linked content");
                    }
                });
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Select a node to view its content");
                });
            }

            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Select Different").clicked() {
                    Self::open_content_selector::<T>(world, selected_node_id, kind);
                }
                if ui.button("Create New").clicked() {
                    Self::open_content_creator::<T>(world, selected_node_id);
                }
            });
        });
        world.insert_resource(state);
    }

    fn render_named_selector(selector_kind: NamedNodeKind, ui: &mut Ui, world: &mut World) {
        let state = world.resource::<ExplorerState>();
        let selection = world.resource::<ExplorerSelection>();

        let (nodes_to_show, current_parent) = match selector_kind {
            NamedNodeKind::NHouse => (state.named_nodes.get(&NamedNodeKind::NHouse), None),
            NamedNodeKind::NUnit => {
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
                                    cn().reducers
                                        .content_select_link(current_parent.unwrap(), *id)
                                        .notify_error_op();
                                }
                            }
                        }

                        let id = *id;
                        if ui.link(name).clicked() {
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

    fn get_current_selected_node(content_kind: NodeKind, world: &World) -> Option<u64> {
        let selection = world.resource::<ExplorerSelection>();

        let (_parent_kind, relationship) = Self::get_node_relationship(content_kind);

        match relationship {
            NodeRelationship::DirectParent(named_kind) => {
                selection.selected.get(&named_kind).copied()
            }
            NodeRelationship::IndirectParent(named_kind) => {
                selection.selected.get(&named_kind).copied()
            }
        }
    }

    fn get_node_relationship(content_kind: NodeKind) -> (NodeKind, NodeRelationship) {
        match content_kind {
            NodeKind::NHouseColor => (
                NodeKind::NHouse,
                NodeRelationship::DirectParent(NamedNodeKind::NHouse),
            ),
            NodeKind::NUnitDescription => (
                NodeKind::NUnit,
                NodeRelationship::DirectParent(NamedNodeKind::NUnit),
            ),
            NodeKind::NUnitBehavior => (
                NodeKind::NUnitDescription,
                NodeRelationship::IndirectParent(NamedNodeKind::NUnit),
            ),
            NodeKind::NUnitRepresentation => (
                NodeKind::NUnitDescription,
                NodeRelationship::IndirectParent(NamedNodeKind::NUnit),
            ),
            NodeKind::NUnitStats => (
                NodeKind::NUnit,
                NodeRelationship::DirectParent(NamedNodeKind::NUnit),
            ),
            NodeKind::NStatusDescription => (
                NodeKind::NStatusMagic,
                NodeRelationship::DirectParent(NamedNodeKind::NStatusMagic),
            ),
            NodeKind::NStatusBehavior => (
                NodeKind::NStatusDescription,
                NodeRelationship::IndirectParent(NamedNodeKind::NStatusMagic),
            ),
            NodeKind::NStatusRepresentation => (
                NodeKind::NStatusMagic,
                NodeRelationship::DirectParent(NamedNodeKind::NStatusMagic),
            ),
            NodeKind::NAbilityDescription => (
                NodeKind::NAbilityMagic,
                NodeRelationship::DirectParent(NamedNodeKind::NAbilityMagic),
            ),
            NodeKind::NAbilityEffect => (
                NodeKind::NAbilityDescription,
                NodeRelationship::IndirectParent(NamedNodeKind::NAbilityMagic),
            ),
            _ => (
                content_kind,
                NodeRelationship::DirectParent(NamedNodeKind::NUnit),
            ),
        }
    }

    fn open_content_selector<T: Node + Component + FTitle>(
        world: &mut World,
        parent_id: Option<u64>,
        content_kind: NodeKind,
    ) {
        let parent_id = match parent_id {
            Some(id) => id,
            None => return,
        };

        let kind = T::kind_s();

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

        let selection = world.resource::<ExplorerSelection>();
        let current_selected = selection
            .content_current
            .get(&(content_kind, parent_id))
            .copied();
        let user_selected = selection
            .content_selected
            .get(&(content_kind, parent_id))
            .copied();

        Confirmation::new("Select Content")
            .content(move |ui, world| {
                let mut selected = None;
                Context::from_world(world, |context| {
                    let _filter_id = ui.id().with("content_selector_filter");

                    for node in &nodes {
                        let node_id = node.id();
                        ui.horizontal(|ui| {
                            node.as_title().compose(context, ui);

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
                });

                if let Some(id) = selected {
                    cn().reducers
                        .content_select_link(parent_id, id)
                        .notify_error(world);

                    world
                        .resource_mut::<ExplorerSelection>()
                        .content_selected
                        .insert((content_kind, parent_id), id);

                    return true;
                }
                false
            })
            .push(world);
    }

    fn open_content_creator<T: Node + Component + Default>(
        world: &mut World,
        parent_id: Option<u64>,
    ) {
        if let Some(_parent_id) = parent_id {
            Confirmation::new("Create New Content")
                .content(move |ui, _world| {
                    ui.label(format!("Create new {}", T::kind_s()));
                    false
                })
                .push(world);
        }
    }
}

impl ExplorerState {
    pub fn refresh_all_caches(&mut self) {
        self.refresh_named_cache(NamedNodeKind::NHouse);
        self.refresh_named_cache(NamedNodeKind::NUnit);
        self.refresh_named_cache(NamedNodeKind::NAbilityMagic);
        self.refresh_named_cache(NamedNodeKind::NStatusMagic);

        self.refresh_content_cache();
        self.refresh_active_player();
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

    fn refresh_active_player(&mut self) {
        // For now, just clear the player ID cache
        // TODO: Implement when player identity is available
        self.active_player_id = None;
    }

    fn cache_player_selection(&mut self, content_kind: NodeKind) {
        if let Some(player_id) = self.active_player_id {
            let selected = cn()
                .db()
                .player_link_selections()
                .iter()
                .find(|s| s.player_id == player_id && s.kind == content_kind.as_ref())
                .map(|s| s.selected_link_id);

            self.player_selections_cache.insert(content_kind, selected);
        }
    }

    fn get_player_selected_node(&self, content_kind: NodeKind) -> Option<u64> {
        self.player_selections_cache
            .get(&content_kind)
            .copied()
            .flatten()
    }
}

#[derive(Clone, Copy, Debug)]
enum NodeRelationship {
    DirectParent(NamedNodeKind),
    IndirectParent(NamedNodeKind),
}
