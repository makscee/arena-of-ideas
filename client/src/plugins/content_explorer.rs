use super::*;

#[derive(Resource, Default)]
pub struct TempNodeStorage(HashMap<String, String>);

impl TempNodeStorage {
    pub fn insert(&mut self, key: String, value: String) {
        self.0.insert(key, value);
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.0.get(key)
    }
}

#[derive(Resource, Default, Clone)]
pub struct ContentExplorerData {
    pub selected_unit: Option<u64>,
    pub owner_filter: OwnerFilter,
    pub units: Vec<u64>,
    pub linked_representations: Vec<(u64, Option<i32>)>,
    pub linked_descriptions: Vec<(u64, Option<i32>)>,
    pub linked_behaviors: Vec<(u64, Option<i32>)>,
    pub linked_stats: Vec<(u64, Option<i32>)>,
    pub linked_houses: Vec<(u64, Option<i32>)>,
    pub needs_refresh: bool,
    pub cached_all_representations: Vec<(u64, Option<i32>)>,
    pub cached_all_descriptions: Vec<(u64, Option<i32>)>,
    pub cached_all_behaviors: Vec<(u64, Option<i32>)>,
    pub cached_all_stats: Vec<(u64, Option<i32>)>,
    pub cached_all_houses: Vec<(u64, Option<i32>)>,

    // View modes for each pane
    pub representations_view_mode: ViewMode,
    pub descriptions_view_mode: ViewMode,
    pub behaviors_view_mode: ViewMode,
    pub stats_view_mode: ViewMode,
    pub houses_view_mode: ViewMode,

    // Current selections for each component type
    pub current_representation: Option<u64>,
    pub current_description: Option<u64>,
    pub current_behavior: Option<u64>,
    pub current_stats: Option<u64>,
}

#[derive(PartialEq, Eq, Clone, Copy, Default)]
pub enum ViewMode {
    #[default]
    Current,
    All,
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

pub struct ContentExplorerPlugin;

impl Plugin for ContentExplorerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::ContentExplorer), Self::init)
            .add_systems(
                Update,
                Self::check_for_refresh.run_if(in_state(GameState::ContentExplorer)),
            )
            .init_resource::<TempNodeStorage>();
    }
}

impl ContentExplorerPlugin {
    pub fn init(world: &mut World) {
        let mut data = ContentExplorerData::default();
        data.owner_filter = OwnerFilter::Content;
        data.needs_refresh = false;
        Self::load_units(&mut data);
        Self::load_all_cached_data(&mut data);

        if let Some(&first_unit) = data.units.first() {
            data.selected_unit = Some(first_unit);
            Self::load_linked_data(&mut data, first_unit);
        }

        world.insert_resource(data);
    }

    fn check_for_refresh(world: &mut World) {
        let needs_refresh = {
            let data = world.resource::<ContentExplorerData>();
            data.needs_refresh
        };

        if needs_refresh {
            let mut data = world.resource_mut::<ContentExplorerData>();
            data.needs_refresh = false;
            Self::load_units(&mut data);
            Self::load_all_cached_data(&mut data);

            if let Some(selected_unit) = data.selected_unit {
                Self::load_linked_data(&mut data, selected_unit);
            }
        }
    }

    fn load_all_cached_data(data: &mut ContentExplorerData) {
        data.cached_all_representations =
            Self::get_all_nodes_of_kind(NodeKind::NUnitRepresentation, data.owner_filter);
        data.cached_all_descriptions =
            Self::get_all_nodes_of_kind(NodeKind::NUnitDescription, data.owner_filter);
        data.cached_all_behaviors =
            Self::get_all_nodes_of_kind(NodeKind::NUnitBehavior, data.owner_filter);
        data.cached_all_stats =
            Self::get_all_nodes_of_kind(NodeKind::NUnitStats, data.owner_filter);
        data.cached_all_houses = Self::get_all_nodes_of_kind(NodeKind::NHouse, data.owner_filter);
    }

    fn load_units(data: &mut ContentExplorerData) {
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

    fn load_linked_data(data: &mut ContentExplorerData, unit_id: u64) {
        data.linked_representations =
            Self::get_linked_nodes(unit_id, NodeKind::NUnitRepresentation);
        data.linked_descriptions = Self::get_linked_nodes(unit_id, NodeKind::NUnitDescription);
        data.linked_behaviors = Self::get_linked_nodes(unit_id, NodeKind::NUnitBehavior);
        data.linked_stats = Self::get_linked_nodes(unit_id, NodeKind::NUnitStats);
        data.linked_houses = Self::get_linked_nodes(unit_id, NodeKind::NHouse);

        // Set current selections to first linked item if none selected
        if data.current_representation.is_none() {
            data.current_representation = data.linked_representations.first().map(|(id, _)| *id);
        }
        if data.current_description.is_none() {
            data.current_description = data.linked_descriptions.first().map(|(id, _)| *id);
        }
        if data.current_behavior.is_none() {
            data.current_behavior = data.linked_behaviors.first().map(|(id, _)| *id);
        }
        if data.current_stats.is_none() {
            data.current_stats = data.linked_stats.first().map(|(id, _)| *id);
        }
    }

    fn get_linked_nodes(unit_id: u64, target_kind: NodeKind) -> Vec<(u64, Option<i32>)> {
        let target_kind_str = target_kind.as_ref();
        let mut nodes = Vec::new();

        for link in cn().db.node_links().iter() {
            if link.parent == unit_id && link.child_kind == target_kind_str {
                nodes.push((link.child, Some(link.rating)));
            } else if link.child == unit_id && link.parent_kind == target_kind_str {
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

    fn generic_merged_pane<T: FTitle + Node + FEdit + FDisplay>(
        ui: &mut Ui,
        world: &mut World,
        pane_name: &str,
        kind: NodeKind,
        linked_nodes: &[(u64, Option<i32>)],
        cached_all_nodes: &[(u64, Option<i32>)],
        selected_unit: Option<u64>,
        view_mode: ViewMode,
        current_selection: Option<u64>,
    ) -> Result<(ViewMode, Option<u64>), ExpressionError> {
        let mut new_view_mode = view_mode;
        let mut new_current_selection = current_selection;

        ui.horizontal(|ui| {
            pane_name.cstr_s(CstrStyle::Heading2).label(ui);

            let all_count = cached_all_nodes.len();
            let mode_text = match view_mode {
                ViewMode::Current => format!("Show All ({})", all_count),
                ViewMode::All => "Show Current".to_string(),
            };

            if ui.button(mode_text).clicked() {
                new_view_mode = match view_mode {
                    ViewMode::Current => ViewMode::All,
                    ViewMode::All => ViewMode::Current,
                };
            }

            if ui.button("+ Add New").clicked() {
                Self::open_create_node_window::<T>(world);
            }
        });

        match view_mode {
            ViewMode::Current => Self::render_current_view::<T>(ui, world, current_selection)?,
            ViewMode::All => {
                new_current_selection = Self::render_all_view::<T>(
                    ui,
                    world,
                    linked_nodes,
                    cached_all_nodes,
                    selected_unit,
                    current_selection,
                )?;
            }
        }

        Ok((new_view_mode, new_current_selection))
    }

    fn render_current_view<T: FTitle + FDisplay + Node>(
        ui: &mut Ui,
        world: &mut World,
        current_selection: Option<u64>,
    ) -> Result<(), ExpressionError> {
        Context::from_world_r(world, |context| {
            if let Some(node_id) = current_selection {
                if let Ok(node) = context.component_by_id::<T>(node_id) {
                    ui.horizontal(|ui| {
                        if ui.button("Edit").clicked() {
                            // TODO: Implement editing functionality
                        }
                    });
                    ui.separator();
                    node.display(context, ui);
                    return Ok(());
                }
            }
            ui.label("No item selected");
            Ok(())
        })
    }

    fn render_all_view<T: FTitle + Node + FEdit>(
        ui: &mut Ui,
        world: &mut World,
        linked_nodes: &[(u64, Option<i32>)],
        cached_all_nodes: &[(u64, Option<i32>)],
        selected_unit: Option<u64>,
        current_selection: Option<u64>,
    ) -> Result<Option<u64>, ExpressionError> {
        let mut new_current_selection = current_selection;
        Context::from_world_r(world, |context| {
            if selected_unit.is_none() {
                ui.label("No unit selected");
                return Ok(());
            }

            let linked_ids: HashSet<u64> = linked_nodes.iter().map(|(id, _)| *id).collect();

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
                        let response = node.title(context).button(ui);
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
                    |context, (node, _, _, _, _, _)| {
                        Ok(VarValue::String(node.title(context).to_string()))
                    },
                )
                .column(
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
                .column(
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
                                                if let Some(selected) = selected_unit {
                                                    if let Err(e) = cn().reducers.content_vote_link(
                                                        selected, *node_id, false,
                                                    ) {
                                                        error!("Failed to vote down link: {}", e);
                                                    }
                                                }
                                            }
                                            if "[green [b +]]".cstr().button(ui).clicked() {
                                                if let Some(selected) = selected_unit {
                                                    if let Err(e) = cn()
                                                        .reducers
                                                        .content_vote_link(selected, *node_id, true)
                                                    {
                                                        error!("Failed to vote up link: {}", e);
                                                    }
                                                }
                                            }
                                        });
                                    });
                                });
                            }
                        } else {
                            if ui.button("Link").clicked() {
                                if let Some(selected) = selected_unit {
                                    if let Err(e) =
                                        cn().reducers.content_vote_link(selected, *node_id, true)
                                    {
                                        error!("Failed to create link: {}", e);
                                    }
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

    pub fn pane_units_list(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        let mut data = world.resource::<ContentExplorerData>().clone();

        ui.horizontal(|ui| {
            "Units".cstr_s(CstrStyle::Heading2).label(ui);
            if ui.button("+ Add New").clicked() {
                Self::open_create_node_window::<NUnit>(world);
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
                    Self::load_units(&mut data);
                    if let Some(selected) = data.selected_unit {
                        Self::load_linked_data(&mut data, selected);
                    }
                }
            }
        });

        ui.separator();

        Context::from_world_r(world, |context| {
            let items: Vec<_> = data
                .units
                .iter()
                .filter_map(|unit_id| {
                    context.component_by_id::<NUnit>(*unit_id).ok().map(|unit| {
                        let rating = cn()
                            .db
                            .nodes_world()
                            .id()
                            .find(unit_id)
                            .map(|node| node.rating)
                            .unwrap_or_default();
                        (unit, *unit_id, rating)
                    })
                })
                .collect();

            items
                .table()
                .column(
                    "Name",
                    |context, ui, (unit, unit_id, _), _| {
                        let is_selected = data.selected_unit == Some(*unit_id);
                        let response = unit.title(context).button(ui);

                        if response.clicked() {
                            data.selected_unit = Some(*unit_id);
                            Self::load_linked_data(&mut data, *unit_id);
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
                    |context, (unit, _, _)| Ok(VarValue::String(unit.title(context).to_string())),
                )
                .column(
                    "Rating",
                    |_context, ui, (_, unit_id, _rating), value| {
                        if let VarValue::i32(rating_val) = value {
                            let response = rating_val.to_string().button(ui);
                            response.bar_menu(|ui| {
                                ui.vertical(|ui| {
                                    "Rating".cstr().label(ui);
                                    ui.horizontal(|ui| {
                                        if "[red [b -]]".cstr().button(ui).clicked() {
                                            if let Err(e) =
                                                cn().reducers.content_vote_node(*unit_id, false)
                                            {
                                                error!("Failed to vote down unit: {}", e);
                                            }
                                        }
                                        if "[green [b +]]".cstr().button(ui).clicked() {
                                            if let Err(e) =
                                                cn().reducers.content_vote_node(*unit_id, true)
                                            {
                                                error!("Failed to vote up unit: {}", e);
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
                    |_context, ui, (_, _unit_id, _), value| {
                        if let VarValue::u64(id) = value {
                            format!("#{}", id).cstr_c(Color32::GRAY).label(ui);
                        }
                        Ok(())
                    },
                    |_context, (_, unit_id, _)| Ok(VarValue::u64(*unit_id)),
                )
                .default_sort(1, false) // Sort by rating descending
                .ui(context, ui);

            Ok(())
        })?;

        world.insert_resource(data);
        Ok(())
    }

    pub fn pane_representations(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        let mut data = world.resource::<ContentExplorerData>().clone();
        let (new_view_mode, new_current) = Self::generic_merged_pane::<NUnitRepresentation>(
            ui,
            world,
            "Representations",
            NodeKind::NUnitRepresentation,
            &data.linked_representations,
            &data.cached_all_representations,
            data.selected_unit,
            data.representations_view_mode,
            data.current_representation,
        )?;
        data.representations_view_mode = new_view_mode;
        data.current_representation = new_current;
        world.insert_resource(data);
        Ok(())
    }

    pub fn pane_descriptions(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        let mut data = world.resource::<ContentExplorerData>().clone();
        let (new_view_mode, new_current) = Self::generic_merged_pane::<NUnitDescription>(
            ui,
            world,
            "Descriptions",
            NodeKind::NUnitDescription,
            &data.linked_descriptions,
            &data.cached_all_descriptions,
            data.selected_unit,
            data.descriptions_view_mode,
            data.current_description,
        )?;
        data.descriptions_view_mode = new_view_mode;
        data.current_description = new_current;
        world.insert_resource(data);
        Ok(())
    }

    pub fn pane_behaviors(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        let mut data = world.resource::<ContentExplorerData>().clone();
        let (new_view_mode, new_current) = Self::generic_merged_pane::<NUnitBehavior>(
            ui,
            world,
            "Behaviors",
            NodeKind::NUnitBehavior,
            &data.linked_behaviors,
            &data.cached_all_behaviors,
            data.selected_unit,
            data.behaviors_view_mode,
            data.current_behavior,
        )?;
        data.behaviors_view_mode = new_view_mode;
        data.current_behavior = new_current;
        world.insert_resource(data);
        Ok(())
    }

    pub fn pane_stats(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        let mut data = world.resource::<ContentExplorerData>().clone();
        let (new_view_mode, new_current) = Self::generic_merged_pane::<NUnitStats>(
            ui,
            world,
            "Stats",
            NodeKind::NUnitStats,
            &data.linked_stats,
            &data.cached_all_stats,
            data.selected_unit,
            data.stats_view_mode,
            data.current_stats,
        )?;
        data.stats_view_mode = new_view_mode;
        data.current_stats = new_current;
        world.insert_resource(data);
        Ok(())
    }

    pub fn pane_houses(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        let mut data = world.resource::<ContentExplorerData>().clone();
        let (new_view_mode, _) = Self::generic_merged_pane::<NHouse>(
            ui,
            world,
            "Houses",
            NodeKind::NHouse,
            &data.linked_houses,
            &data.cached_all_houses,
            data.selected_unit,
            data.houses_view_mode,
            None, // Houses don't have a current selection
        )?;
        data.houses_view_mode = new_view_mode;
        world.insert_resource(data);
        Ok(())
    }

    fn open_create_node_window<T: Node + FEdit + StringData + Default>(world: &mut World) {
        let kind = T::kind_s();
        let window_id = format!("create_{}", kind.as_ref());

        if WindowPlugin::is_open(&window_id, world) {
            return;
        }

        Window::new(window_id.clone(), move |ui, world| {
            Self::create_node_content::<T>(ui, world, window_id.clone())
        })
        .default_width(600.0)
        .default_height(400.0)
        .push(world);
    }

    fn create_node_content<T: Node + FEdit + StringData + Default>(
        ui: &mut Ui,
        world: &mut World,
        window_id: String,
    ) {
        let kind = T::kind_s();
        ui.vertical(|ui| {
            ui.heading(format!("Create New {}", kind.as_ref()));
            ui.separator();

            if let Ok(mut node) = Self::get_or_create_temp_node::<T>(world, &window_id) {
                let changed = Context::from_world_r(world, |context| Ok(node.edit(context, ui)))
                    .unwrap_or(false);

                if changed {
                    Self::set_temp_node(world, &window_id, node);
                }
            }

            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Publish").clicked() {
                    if let Some(packed_data) = Self::get_temp_node_data(world, &window_id) {
                        let mut pack = PackedNodes::default();
                        pack.root = 1;
                        pack.add_node(kind.to_string(), packed_data, 1);
                        let pack_string = to_ron_string(&pack);

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
        world: &World,
        window_id: &str,
    ) -> Result<T, ExpressionError> {
        let storage_key = format!("temp_node_{}", window_id);

        if let Some(stored_data) = world
            .get_resource::<TempNodeStorage>()
            .and_then(|storage| storage.get(&storage_key))
        {
            let mut node = T::default();
            node.inject_data(stored_data)?;
            Ok(node)
        } else {
            Ok(T::default())
        }
    }

    fn set_temp_node<T: Node + StringData>(world: &mut World, window_id: &str, node: T) {
        let storage_key = format!("temp_node_{}", window_id);
        let data = node.get_data();

        if let Some(mut storage) = world.get_resource_mut::<TempNodeStorage>() {
            storage.insert(storage_key, data);
        } else {
            let mut storage = TempNodeStorage::default();
            storage.insert(storage_key, data);
            world.insert_resource(storage);
        }
    }

    fn get_temp_node_data(world: &World, window_id: &str) -> Option<String> {
        let storage_key = format!("temp_node_{}", window_id);
        world
            .get_resource::<TempNodeStorage>()?
            .get(&storage_key)
            .cloned()
    }
}
