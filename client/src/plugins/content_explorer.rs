use super::*;

#[derive(Clone, Default)]
pub struct EditState {
    pub is_editing: bool,
    pub has_changes: bool,
    pub original_data: String,
    pub current_data: String,
}

#[derive(Clone, Default)]
pub struct NodeData {
    pub linked_nodes: Vec<(u64, Option<i32>)>,
    pub cached_all_nodes: Vec<(u64, Option<i32>)>,
    pub current_selection: Option<u64>,
}

#[derive(Resource, Default, Clone)]
pub struct ExplorerData {
    pub selected_unit: Option<u64>,
    pub selected_house: Option<u64>,
    pub owner_filter: OwnerFilter,
    pub units: Vec<u64>,
    pub houses: Vec<u64>,
    pub node_data: HashMap<NodeKind, NodeData>,
    pub needs_refresh: bool,
    pub node_colors: HashMap<u64, Color32>,
}

impl ExplorerData {
    pub fn get_node_data(&mut self, kind: NodeKind) -> &mut NodeData {
        self.node_data.entry(kind).or_default()
    }
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
    pub fn should_include(self, owner: u64) -> bool {
        match self {
            OwnerFilter::All => true,
            OwnerFilter::Core => owner == ID_CORE,
            OwnerFilter::Content => owner == 0 || owner == ID_CORE,
        }
    }
}

pub struct ExplorerPlugin;

impl Plugin for ExplorerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Explorer), Self::init)
            .add_systems(
                Update,
                Self::check_for_refresh.run_if(in_state(GameState::Explorer)),
            );
    }
}

impl ExplorerPlugin {
    pub fn init(world: &mut World) {
        let mut data = ExplorerData::default();
        data.owner_filter = OwnerFilter::Content;
        data.needs_refresh = false;

        Self::load_units(&mut data);
        Self::load_houses(&mut data);
        Self::load_all_cached_data(&mut data);

        let selected_unit = data.units.first().copied();
        let selected_house = data.houses.first().copied();

        if let Some(first_unit) = selected_unit {
            data.selected_unit = Some(first_unit);
            Self::load_unit_linked_data(&mut data, first_unit);
        }

        if let Some(first_house) = selected_house {
            data.selected_house = Some(first_house);
            Self::load_house_linked_data(&mut data, first_house);
        }

        world.insert_resource(data);
        Self::load_node_colors_with_world(world);
    }

    fn check_for_refresh(world: &mut World) {
        let needs_refresh = {
            let data = world.resource::<ExplorerData>();
            data.needs_refresh
        };

        if needs_refresh {
            Self::reload_explorer_data(world);
        }
    }

    fn reload_explorer_data(world: &mut World) {
        let mut data = world.resource_mut::<ExplorerData>();
        data.needs_refresh = false;
        Self::load_units(&mut data);
        Self::load_houses(&mut data);
        Self::load_all_cached_data(&mut data);

        Self::load_node_colors_with_world(world);

        let mut data = world.resource_mut::<ExplorerData>();
        if let Some(selected_unit) = data.selected_unit {
            Self::load_unit_linked_data(&mut data, selected_unit);
        }

        if let Some(selected_house) = data.selected_house {
            Self::load_house_linked_data(&mut data, selected_house);
        }
    }

    fn load_all_cached_data(data: &mut ExplorerData) {
        let kinds = [
            NodeKind::NUnitRepresentation,
            NodeKind::NUnitDescription,
            NodeKind::NUnitBehavior,
            NodeKind::NUnitStats,
            NodeKind::NHouse,
            NodeKind::NHouseColor,
            NodeKind::NAbilityMagic,
            NodeKind::NStatusMagic,
            NodeKind::NUnit,
        ];

        for kind in kinds {
            data.get_node_data(kind).cached_all_nodes =
                Self::get_all_nodes_of_kind(kind, data.owner_filter);
        }
    }

    fn load_units(data: &mut ExplorerData) {
        data.units = cn()
            .db
            .nodes_world()
            .iter()
            .filter(|node| node.kind == "NUnit" && data.owner_filter.should_include(node.owner))
            .map(|node| node.id)
            .collect();

        data.units.sort_by_key(|id| {
            cn().db
                .nodes_world()
                .id()
                .find(id)
                .map(|node| node.rating)
                .unwrap_or_default()
        });
        data.units.reverse();
    }

    fn load_houses(data: &mut ExplorerData) {
        data.houses = cn()
            .db
            .nodes_world()
            .iter()
            .filter(|node| {
                node.kind == NodeKind::NHouse.as_ref()
                    && data.owner_filter.should_include(node.owner)
            })
            .map(|node| node.id)
            .collect();

        data.houses.sort_by_key(|id| {
            cn().db
                .nodes_world()
                .id()
                .find(id)
                .map(|node| node.rating)
                .unwrap_or_default()
        });
        data.houses.reverse();
    }

    fn load_unit_linked_data(data: &mut ExplorerData, unit_id: u64) {
        let unit_kinds = [
            NodeKind::NUnitRepresentation,
            NodeKind::NUnitDescription,
            NodeKind::NUnitBehavior,
            NodeKind::NUnitStats,
            NodeKind::NHouse,
        ];

        for kind in unit_kinds {
            let node_data = data.get_node_data(kind);
            node_data.linked_nodes = Self::get_linked_nodes(unit_id, kind);
            node_data.current_selection = node_data.linked_nodes.first().map(|(id, _)| *id);
        }
    }

    fn load_house_linked_data(data: &mut ExplorerData, house_id: u64) {
        let house_kinds = [
            NodeKind::NHouseColor,
            NodeKind::NAbilityMagic,
            NodeKind::NStatusMagic,
            NodeKind::NUnit,
        ];

        for kind in house_kinds {
            let node_data = data.get_node_data(kind);
            node_data.linked_nodes = Self::get_linked_nodes(house_id, kind);
            node_data.current_selection = node_data.linked_nodes.first().map(|(id, _)| *id);
        }
    }

    fn get_linked_nodes(node_id: u64, target_kind: NodeKind) -> Vec<(u64, Option<i32>)> {
        let target_kind_str = target_kind.as_ref();
        let mut nodes = Vec::new();

        for link in cn().db.node_links().iter() {
            if link.parent == node_id && link.child_kind == target_kind_str {
                nodes.push((link.child, Some(link.rating)));
            } else if link.child == node_id && link.parent_kind == target_kind_str {
                nodes.push((link.parent, Some(link.rating)));
            }
        }

        nodes.sort_by(|a, b| b.1.unwrap_or_default().cmp(&a.1.unwrap_or_default()));
        nodes
    }

    fn get_all_nodes_of_kind(kind: NodeKind, owner_filter: OwnerFilter) -> Vec<(u64, Option<i32>)> {
        let kind_str = kind.as_ref();
        let mut nodes = cn()
            .db
            .nodes_world()
            .iter()
            .filter(|node| node.kind == kind_str && owner_filter.should_include(node.owner))
            .map(|node| (node.id, Some(node.rating)))
            .collect::<Vec<_>>();

        nodes.sort_by(|a, b| b.1.unwrap_or_default().cmp(&a.1.unwrap_or_default()));
        nodes
    }

    fn generic_merged_pane<T: FTitle + Node + FEdit + FDisplay + FCompactView + StringData>(
        ui: &mut Ui,
        world: &mut World,
        linked_nodes: &[(u64, Option<i32>)],
        cached_all_nodes: &[(u64, Option<i32>)],
        selected_main_node: Option<u64>,
        mut current_selection: Option<u64>,
    ) -> Result<Option<u64>, ExpressionError> {
        ui.horizontal(|ui| {
            if let Some(node_id) = current_selection {
                let edit_id = Id::new("edit_state").with(node_id);
                let mut edit_state = ui
                    .ctx()
                    .data_mut(|d| d.get_temp_mut_or(edit_id, EditState::default()).clone());

                if ui.button("Edit").clicked() && !edit_state.is_editing {
                    edit_state.is_editing = true;
                    let _ = Context::from_world_r(world, |context| {
                        if let Ok(node) = context.component_by_id::<T>(node_id) {
                            edit_state.original_data = node.get_data();
                            edit_state.current_data = node.get_data();
                            edit_state.has_changes = false;
                        }
                        Ok(())
                    });
                }

                ui.ctx().data_mut(|d| d.insert_temp(edit_id, edit_state));
            }

            if ui.button("+ Add New").clicked() {
                Self::open_create_node_window::<T>(world);
            }
        });

        Self::render_current_view::<T>(ui, world, current_selection)?;

        ui.separator();

        current_selection = Self::render_all_view::<T>(
            ui,
            world,
            linked_nodes,
            cached_all_nodes,
            selected_main_node,
            current_selection,
        )?;

        Ok(current_selection)
    }

    fn render_current_view<T: FTitle + FDisplay + Node + FEdit + StringData>(
        ui: &mut Ui,
        world: &mut World,
        current_selection: Option<u64>,
    ) -> Result<(), ExpressionError> {
        let node_colors = world.resource::<ExplorerData>().node_colors.clone();
        Context::from_world_r(world, |context| {
            if let Some(node_id) = current_selection {
                if let Ok(original_node) = context.component_by_id::<T>(node_id) {
                    let edit_id = Id::new("edit_state").with(node_id);

                    let mut edit_state = ui
                        .ctx()
                        .data_mut(|d| d.get_temp_mut_or(edit_id, EditState::default()).clone());

                    if edit_state.is_editing {
                        ui.horizontal(|ui| {
                            let has_changes = edit_state.current_data != edit_state.original_data;
                            let save_button =
                                ui.add_enabled(has_changes, egui::Button::new("Save"));
                            if save_button.clicked() {
                                edit_state.is_editing = false;
                                edit_state.has_changes = true;
                            }
                            if ui.button("Cancel").clicked() {
                                edit_state.is_editing = false;
                                edit_state.current_data = edit_state.original_data.clone();
                                edit_state.has_changes = false;
                            }
                        });
                    }

                    if edit_state.has_changes && !edit_state.is_editing {
                        ui.horizontal(|ui| {
                            if ui.button("Publish").clicked() {
                                let mut pack = PackedNodes::default();
                                pack.root = 1;
                                pack.add_node(
                                    T::kind_s().to_string(),
                                    edit_state.current_data.clone(),
                                    1,
                                );
                                let pack_string = to_ron_string(&pack);

                                if let Err(e) = cn().reducers.content_publish_node(pack_string) {
                                    error!("Failed to publish node: {}", e);
                                } else {
                                    info!("Published edited {} node", T::kind_s().as_ref());
                                    edit_state.has_changes = false;
                                }
                            }
                        });
                    }

                    if edit_state.is_editing {
                        let mut temp_node = original_node.clone();
                        if temp_node.inject_data(&edit_state.current_data).is_ok() {
                            let changed = temp_node.edit(context, ui);
                            if changed {
                                edit_state.current_data = temp_node.get_data();
                            }
                        }
                    } else {
                        let display_node = if edit_state.has_changes {
                            let mut temp_node = original_node.clone();
                            if temp_node.inject_data(&edit_state.current_data).is_ok() {
                                temp_node
                            } else {
                                original_node.clone()
                            }
                        } else {
                            original_node.clone()
                        };

                        // Set color context based on cached house color and display
                        if let Some(&color) = node_colors.get(&node_id) {
                            context.with_layer(
                                ContextLayer::Var(VarName::color, VarValue::Color32(color)),
                                |context| {
                                    display_node.display(context, ui);
                                },
                            );
                        } else {
                            display_node.display(context, ui);
                        }
                    }

                    ui.ctx().data_mut(|d| d.insert_temp(edit_id, edit_state));
                    return Ok(());
                }
            }
            ui.label("No item selected");
            Ok(())
        })
    }

    fn render_all_view<T: FTitle + Node + FEdit + FCompactView>(
        ui: &mut Ui,
        world: &mut World,
        linked_nodes: &[(u64, Option<i32>)],
        cached_all_nodes: &[(u64, Option<i32>)],
        selected_main_node: Option<u64>,
        current_selection: Option<u64>,
    ) -> Result<Option<u64>, ExpressionError> {
        let mut new_current_selection = current_selection;
        Context::from_world_r(world, |context| {
            if selected_main_node.is_none() {
                ui.label("No main item selected");
                return Ok(());
            }
            let linked_ids: HashSet<u64> = linked_nodes.iter().map(|(id, _)| *id).collect();
            let node_colors = &context.world()?.resource::<ExplorerData>().node_colors;
            let items: Vec<_> = cached_all_nodes
                .iter()
                .filter_map(|(node_id, node_rating)| {
                    let is_linked = linked_ids.contains(node_id);
                    let is_current = current_selection == Some(*node_id);
                    let link_rating = if is_linked {
                        linked_nodes
                            .iter()
                            .find(|(id, _)| *id == *node_id)
                            .and_then(|(_, rating)| *rating)
                            .unwrap_or_default()
                    } else {
                        0
                    };

                    context.component_by_id::<T>(*node_id).ok().map(|node| {
                        (
                            node,
                            *node_id,
                            node_rating.unwrap_or_default(),
                            link_rating,
                            is_linked,
                            is_current,
                        )
                    })
                })
                .collect();

            items
                .table()
                .column(
                    "Name",
                    |context, ui, (node, node_id, _, _, _, is_current), _| {
                        let response = context.with_layer_ref_r(
                            ContextLayer::Var(
                                VarName::color,
                                VarValue::Color32(
                                    node_colors.get(node_id).copied().unwrap_or(MISSING_COLOR),
                                ),
                            ),
                            |context| Ok(node.render(context).compact_view_button(ui)),
                        )?;

                        if response.clicked() {
                            new_current_selection = Some(*node_id);
                        }
                        if *is_current {
                            ui.painter().rect_stroke(
                                response.rect.expand(2.0),
                                0.0,
                                Stroke::new(2.0, GREEN),
                                egui::StrokeKind::Middle,
                            );
                        }
                        Ok(())
                    },
                    |context, (node, _node_id, _, _, _, _)| {
                        let title = node.title(context).to_string();
                        Ok(VarValue::String(title))
                    },
                )
                .column_with_hover_text(
                    "⭐",
                    "Node Rating",
                    |_context, ui, (_, node_id, _node_rating, _, _, _), value| {
                        if let VarValue::i32(rating) = value {
                            let response = rating.to_string().button(ui);
                            response.bar_menu(|ui| {
                                ui.vertical(|ui| {
                                    "Node Rating".cstr().label(ui);
                                    ui.horizontal(|ui| {
                                        if "[red [b -]]".cstr().button(ui).clicked() {
                                            if let Err(e) =
                                                cn().reducers.content_vote_node(*node_id, false)
                                            {
                                                error!("Failed to vote down node: {}", e);
                                            }
                                        }
                                        if "[green [b +]]".cstr().button(ui).clicked() {
                                            if let Err(e) =
                                                cn().reducers.content_vote_node(*node_id, true)
                                            {
                                                error!("Failed to vote up node: {}", e);
                                            }
                                        }
                                    });
                                });
                            });
                        }
                        Ok(())
                    },
                    |_context, (_, _, node_rating, _, _, _)| Ok(VarValue::i32(*node_rating)),
                )
                .column_with_hover_text(
                    "🔗",
                    "Link Rating",
                    |_context, ui, (_, node_id, _, _link_rating, is_linked, _), value| {
                        if *is_linked {
                            if let VarValue::i32(rating) = value {
                                let response = rating.to_string().button(ui);
                                response.bar_menu(|ui| {
                                    ui.vertical(|ui| {
                                        "Link Rating".cstr().label(ui);
                                        ui.horizontal(|ui| {
                                            if "[red [b -]]".cstr().button(ui).clicked() {
                                                if let Some(selected) = selected_main_node {
                                                    // if let Err(e) = cn().reducers.content_vote_link(
                                                    //     selected, *node_id, false,
                                                    // ) {
                                                    //     error!("Failed to vote down link: {}", e);
                                                    // }
                                                }
                                            }
                                            if "[green [b +]]".cstr().button(ui).clicked() {
                                                if let Some(selected) = selected_main_node {
                                                    // if let Err(e) = cn()
                                                    //     .reducers
                                                    //     .content_vote_link(selected, *node_id, true)
                                                    // {
                                                    //     error!("Failed to vote up link: {}", e);
                                                    // }
                                                }
                                            }
                                        });
                                    });
                                });
                            }
                        } else {
                            if ui.button("Link").clicked() {
                                if let Some(selected) = selected_main_node {
                                    // if let Err(e) =
                                    //     cn().reducers.content_vote_link(selected, *node_id, true)
                                    // {
                                    //     error!("Failed to create link: {}", e);
                                    // }
                                }
                            }
                        }
                        Ok(())
                    },
                    |_context, (_, _, _, link_rating, is_linked, _)| {
                        if *is_linked {
                            Ok(VarValue::i32(*link_rating))
                        } else {
                            Ok(VarValue::i32(0))
                        }
                    },
                )
                .column(
                    "Status",
                    |_context, ui, (_, _, _, _, is_linked, is_current), _| {
                        if *is_current {
                            "[green Current]".cstr().label(ui);
                        } else if *is_linked {
                            "[yellow Linked]".cstr().label(ui);
                        } else {
                            "[tw Available]".cstr().label(ui);
                        }
                        Ok(())
                    },
                    |_context, (_, _, _, _, is_linked, is_current)| {
                        Ok(VarValue::bool(*is_linked && *is_current))
                    },
                )
                .default_sort(1, false)
                .ui(context, ui);

            Ok(())
        })?;

        Ok(new_current_selection)
    }

    pub fn pane_generic_list<T: FTitle + Node + FEdit + FDisplay + FCompactView + StringData>(
        ui: &mut Ui,
        world: &mut World,
        title: &str,
        node_list: &[u64],
        selected_node: Option<u64>,
        on_select: impl Fn(&mut ExplorerData, u64) + Send + Sync,
    ) -> Result<(), ExpressionError> {
        let mut data = world.remove_resource::<ExplorerData>().unwrap();
        let node_colors = data.node_colors.clone();

        ui.horizontal(|ui| {
            title.cstr_s(CstrStyle::Heading2).label(ui);
            if ui.button("+ Add New").clicked() {
                Self::open_create_node_window::<T>(world);
            }
        });

        ui.horizontal(|ui| {
            ui.label("Filter:");
            for filter in OwnerFilter::iter() {
                if ui
                    .selectable_label(data.owner_filter == filter, filter.cstr())
                    .clicked()
                {
                    data.owner_filter = filter;
                    if title == "Units" {
                        Self::load_units(&mut data);
                        if let Some(selected) = data.selected_unit {
                            Self::load_unit_linked_data(&mut data, selected);
                        }
                    } else if title == "Houses" {
                        Self::load_houses(&mut data);
                        if let Some(selected) = data.selected_house {
                            Self::load_house_linked_data(&mut data, selected);
                        }
                    }
                }
            }
        });

        ui.separator();

        Context::from_world_r(world, |context| {
            let items: Vec<_> = node_list
                .iter()
                .filter_map(|node_id| {
                    context.component_by_id::<T>(*node_id).ok().map(|node| {
                        let rating = cn()
                            .db
                            .nodes_world()
                            .id()
                            .find(node_id)
                            .map(|node| node.rating)
                            .unwrap_or_default();
                        (node, *node_id, rating)
                    })
                })
                .collect();

            items
                .table()
                .column(
                    "Name",
                    |context, ui, (node, node_id, _), _| {
                        let is_selected = selected_node == Some(*node_id);
                        let response = context.with_layer_ref_r(
                            ContextLayer::Var(
                                VarName::color,
                                VarValue::Color32(
                                    node_colors.get(node_id).copied().unwrap_or(MISSING_COLOR),
                                ),
                            ),
                            |context| Ok(node.render(context).compact_view_button(ui)),
                        )?;

                        if response.clicked() {
                            on_select(&mut data, *node_id);
                        }

                        if is_selected {
                            ui.painter().rect_stroke(
                                response.rect.expand(2.0),
                                0.0,
                                Stroke::new(2.0, GREEN),
                                egui::StrokeKind::Middle,
                            );
                        }

                        Ok(())
                    },
                    |context, (node, _node_id, _)| {
                        let title = node.title(context).to_string();
                        Ok(VarValue::String(title))
                    },
                )
                .column_with_hover_text(
                    "⭐",
                    "Rating",
                    |_context, ui, (_, node_id, _rating), value| {
                        if let VarValue::i32(rating_val) = value {
                            let response = rating_val.to_string().button(ui);
                            response.bar_menu(|ui| {
                                ui.vertical(|ui| {
                                    "Rating".cstr().label(ui);
                                    ui.horizontal(|ui| {
                                        if "[red [b -]]".cstr().button(ui).clicked() {
                                            if let Err(e) =
                                                cn().reducers.content_vote_node(*node_id, false)
                                            {
                                                error!("Failed to vote down node: {}", e);
                                            }
                                        }
                                        if "[green [b +]]".cstr().button(ui).clicked() {
                                            if let Err(e) =
                                                cn().reducers.content_vote_node(*node_id, true)
                                            {
                                                error!("Failed to vote up node: {}", e);
                                            }
                                        }
                                    });
                                });
                            });
                        }
                        Ok(())
                    },
                    |_context, (_, _, rating)| Ok(VarValue::i32(*rating)),
                )
                .column(
                    "ID",
                    |_context, ui, (_, _node_id, _), value| {
                        if let VarValue::u64(id) = value {
                            format!("#{}", id).cstr_c(Color32::GRAY).label(ui);
                        }
                        Ok(())
                    },
                    |_context, (_, node_id, _)| Ok(VarValue::u64(*node_id)),
                )
                .default_sort(1, false)
                .ui(context, ui);

            Ok(())
        })?;

        world.insert_resource(data);
        Ok(())
    }

    pub fn pane_units_list(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        let data = world.resource::<ExplorerData>();
        let units = data.units.clone();
        let selected = data.selected_unit;

        Self::pane_generic_list::<NUnit>(ui, world, "Units", &units, selected, |data, node_id| {
            data.selected_unit = Some(node_id);
            Self::load_unit_linked_data(data, node_id);
        })
    }

    pub fn pane_houses_list(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        let data = world.resource::<ExplorerData>();
        let houses = data.houses.clone();
        let selected = data.selected_house;

        Self::pane_generic_list::<NHouse>(
            ui,
            world,
            "Houses",
            &houses,
            selected,
            |data, node_id| {
                data.selected_house = Some(node_id);
                Self::load_house_linked_data(data, node_id);
            },
        )
    }

    pub fn pane_node_kind<T: FTitle + Node + FEdit + FDisplay + FCompactView + StringData>(
        ui: &mut Ui,
        world: &mut World,
        kind: NodeKind,
        selected_main_node: Option<u64>,
    ) -> Result<(), ExpressionError> {
        let mut data = world.resource::<ExplorerData>().clone();
        let node_data = data.get_node_data(kind);
        let linked_nodes = node_data.linked_nodes.clone();
        let cached_all_nodes = node_data.cached_all_nodes.clone();
        let current_selection = node_data.current_selection;

        let new_current = Self::generic_merged_pane::<T>(
            ui,
            world,
            &linked_nodes,
            &cached_all_nodes,
            selected_main_node,
            current_selection,
        )?;

        let node_data = data.get_node_data(kind);
        node_data.current_selection = new_current;
        world.insert_resource(data);
        Ok(())
    }

    pub fn pane_representations(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        let selected_unit = world.resource::<ExplorerData>().selected_unit;
        Self::pane_node_kind::<NUnitRepresentation>(
            ui,
            world,
            NodeKind::NUnitRepresentation,
            selected_unit,
        )
    }

    pub fn pane_descriptions(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        let selected_unit = world.resource::<ExplorerData>().selected_unit;
        Self::pane_node_kind::<NUnitDescription>(
            ui,
            world,
            NodeKind::NUnitDescription,
            selected_unit,
        )
    }

    pub fn pane_behaviors(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        let selected_unit = world.resource::<ExplorerData>().selected_unit;
        Self::pane_node_kind::<NUnitBehavior>(ui, world, NodeKind::NUnitBehavior, selected_unit)
    }

    pub fn pane_stats(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        let selected_unit = world.resource::<ExplorerData>().selected_unit;
        Self::pane_node_kind::<NUnitStats>(ui, world, NodeKind::NUnitStats, selected_unit)
    }

    pub fn pane_houses(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        let selected_unit = world.resource::<ExplorerData>().selected_unit;
        Self::pane_node_kind::<NHouse>(ui, world, NodeKind::NHouse, selected_unit)
    }

    pub fn pane_house_colors(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        let selected_house = world.resource::<ExplorerData>().selected_house;
        Self::pane_node_kind::<NHouseColor>(ui, world, NodeKind::NHouseColor, selected_house)
    }

    pub fn pane_ability_magic(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        let selected_house = world.resource::<ExplorerData>().selected_house;
        Self::pane_node_kind::<NAbilityMagic>(ui, world, NodeKind::NAbilityMagic, selected_house)
    }

    pub fn pane_status_magic(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        let selected_house = world.resource::<ExplorerData>().selected_house;
        Self::pane_node_kind::<NStatusMagic>(ui, world, NodeKind::NStatusMagic, selected_house)
    }

    pub fn pane_house_units(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        let selected_house = world.resource::<ExplorerData>().selected_house;
        Self::pane_node_kind::<NUnit>(ui, world, NodeKind::NUnit, selected_house)
    }

    fn open_create_node_window<T: Node + FEdit + StringData + Default>(world: &mut World) {
        let kind = T::kind_s();
        let window_id = format!("Create {}", kind.cstr());
        if WindowPlugin::is_open(&window_id, world) {
            return;
        }
        Window::new(window_id, move |ui, world| {
            Self::create_node_content::<T>(ui, world)
        })
        .default_width(600.0)
        .default_height(400.0)
        .push(world);
    }

    fn create_node_content<T: Node + FEdit + StringData + Default>(ui: &mut Ui, world: &mut World) {
        let kind = T::kind_s();
        ui.vertical(|ui| {
            ui.heading(format!("Create New {}", kind.as_ref()));
            ui.separator();

            if let Ok(mut node) = Self::get_or_create_temp_node::<T>(ui) {
                let changed = Context::from_world_r(world, |context| Ok(node.edit(context, ui)))
                    .unwrap_or(false);

                if changed {
                    Self::set_temp_node(ui, node);
                }
            }

            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Publish").clicked() {
                    if let Ok(node) = Self::get_or_create_temp_node::<T>(ui) {
                        let packed_data = node.get_data();
                        dbg!(&node, &packed_data);
                        let mut pack = PackedNodes::default();
                        pack.root = 1;
                        pack.add_node(kind.to_string(), packed_data, 1);
                        let pack_string = to_ron_string(&pack);
                        dbg!(&pack_string);

                        if let Err(e) = cn().reducers.content_publish_node(pack_string) {
                            error!("Failed to publish node: {}", e);
                        } else {
                            info!("Published new {} node", kind.as_ref());
                            WindowPlugin::close_current(world);
                        }
                    }
                }

                if ui.button("Cancel").clicked() {
                    WindowPlugin::close_current(world);
                }
            });
        });
    }

    fn get_or_create_temp_node<T: Node + Default + StringData>(
        ui: &mut Ui,
    ) -> Result<T, ExpressionError> {
        let storage_id = Self::temp_node_id::<T>();
        let stored_data = ui.ctx().data(|d| d.get_temp::<String>(storage_id));
        if let Some(data) = stored_data {
            let mut node = T::default();
            node.inject_data(&data)?;
            Ok(node)
        } else {
            Ok(T::default())
        }
    }

    fn temp_node_id<T: Node>() -> Id {
        Id::new(T::kind_s()).with("temp_node")
    }

    fn set_temp_node<T: Node + StringData>(ui: &mut Ui, node: T) {
        let storage_id = Self::temp_node_id::<T>();
        let data = node.get_data();
        ui.ctx().data_mut(|d| d.insert_temp(storage_id, data));
    }

    fn load_node_colors_with_world(world: &mut World) {
        let owner_filter = world.resource::<ExplorerData>().owner_filter;

        // Get all nodes that should have colors
        let all_node_ids: Vec<u64> = cn()
            .db
            .nodes_world()
            .iter()
            .filter(|node| {
                matches!(
                    node.kind.to_kind(),
                    NodeKind::NUnit
                        | NodeKind::NHouse
                        | NodeKind::NAbilityMagic
                        | NodeKind::NStatusMagic
                ) && owner_filter.should_include(node.owner)
            })
            .map(|node| node.id)
            .collect();

        let mut node_colors = HashMap::new();
        Context::from_world(world, |context| {
            for node_id in all_node_ids {
                if let Some(color) = Self::get_house_color_for_node_with_context(context, node_id) {
                    node_colors.insert(node_id, color);
                }
            }
        });

        world.resource_mut::<ExplorerData>().node_colors = node_colors;
    }

    fn get_house_color_for_node_with_context(context: &Context, node_id: u64) -> Option<Color32> {
        // Find the house linked to this node
        let house_id = Self::find_linked_house(node_id)?;

        // Find the house color linked to the house
        let house_color_id = Self::find_linked_house_color(house_id)?;

        // Get the actual color from the house color node
        if let Ok(house_color_node) = context.component_by_id::<NHouseColor>(house_color_id) {
            return Some(house_color_node.color.c32());
        }

        None
    }

    fn find_linked_house(node_id: u64) -> Option<u64> {
        let mut links: Vec<(u64, i32)> = default();
        for link in cn().db.node_links().iter() {
            if link.child == node_id && link.parent_kind == NodeKind::NHouse.as_ref() {
                links.push((link.parent, link.rating));
            }
            if link.parent == node_id && link.child_kind == NodeKind::NHouse.as_ref() {
                links.push((link.child, link.rating));
            }
        }
        links.into_iter().max_by_key(|(_, r)| *r).map(|(id, _)| id)
    }

    fn find_linked_house_color(house_id: u64) -> Option<u64> {
        let mut links: Vec<(u64, i32)> = default();
        for link in cn().db.node_links().iter() {
            if link.parent == house_id && link.child_kind == NodeKind::NHouseColor.as_ref() {
                links.push((link.child, link.rating));
            }
        }
        links.into_iter().max_by_key(|(_, r)| *r).map(|(id, _)| id)
    }
}
