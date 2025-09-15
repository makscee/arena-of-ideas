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
            .init_resource::<TempNodeStorage>();
    }
}

impl ContentExplorerPlugin {
    pub fn init(world: &mut World) {
        let mut data = ContentExplorerData::default();
        data.owner_filter = OwnerFilter::Content;
        Self::load_units(&mut data);

        if let Some(&first_unit) = data.units.first() {
            data.selected_unit = Some(first_unit);
            Self::load_linked_data(&mut data, first_unit);
        }

        world.insert_resource(data);
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

    fn generic_pane_current<T: FTitle + FDisplay + Node>(
        ui: &mut Ui,
        world: &mut World,
        linked_nodes: &[(u64, Option<i32>)],
    ) -> Result<(), ExpressionError> {
        Context::from_world_r(world, |context| {
            if let Some((node_id, _)) = linked_nodes.first() {
                if let Ok(node) = context.component_by_id::<T>(*node_id) {
                    node.display(context, ui);
                    return Ok(());
                }
            }
            ui.label("No item selected or linked");
            Ok(())
        })
    }

    fn generic_pane_list<T: FTitle + Node + FEdit>(
        ui: &mut Ui,
        world: &mut World,
        kind: NodeKind,
        linked_nodes: &[(u64, Option<i32>)],
        owner_filter: OwnerFilter,
        selected_unit: Option<u64>,
    ) -> Result<(), ExpressionError> {
        ui.horizontal(|ui| {
            ui.label(format!("{} List", kind.as_ref()));
            if ui.button("+ Add New").clicked() {
                Self::open_create_node_window::<T>(world);
            }
        });
        ui.separator();

        Context::from_world_r(world, |context| {
            if selected_unit.is_none() {
                ui.label("No unit selected");
                return Ok(());
            }

            let all_nodes = Self::get_all_nodes_of_kind(kind, owner_filter);
            let linked_ids: HashSet<u64> = linked_nodes.iter().map(|(id, _)| *id).collect();

            ScrollArea::vertical().show(ui, |ui| {
                for (node_id, node_rating) in &all_nodes {
                    let is_linked = linked_ids.contains(node_id);
                    let link_rating = if is_linked {
                        linked_nodes
                            .iter()
                            .find(|(id, _)| *id == *node_id)
                            .and_then(|(_, rating)| *rating)
                            .unwrap_or_default()
                    } else {
                        0
                    };

                    if let Ok(node) = context.component_by_id::<T>(*node_id) {
                        let title_text = node.title(context).to_string();

                        ui.horizontal(|ui| {
                            ui.label(link_rating.to_string());
                            ui.label(node_rating.map(|r| r.to_string()).unwrap_or_default());

                            if is_linked {
                                ui.colored_label(egui::Color32::YELLOW, title_text);
                            } else {
                                ui.label(title_text);
                            }
                        });
                    }
                }
            });
            Ok(())
        })
    }

    pub fn pane_units_list(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        let mut data = world.resource::<ContentExplorerData>().clone();

        ui.horizontal(|ui| {
            ui.label("Units List");
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

        let selected_unit = data.selected_unit;
        let units = data.units.clone();

        Context::from_world_r(world, |context| {
            ScrollArea::vertical().show(ui, |ui| {
                for unit_id in &units {
                    let is_selected = selected_unit == Some(*unit_id);

                    if let Ok(unit) = context.component_by_id::<NUnit>(*unit_id) {
                        let unit_name = unit.title(context).to_string();
                        let rating = cn()
                            .db
                            .nodes_world()
                            .id()
                            .find(unit_id)
                            .map(|node| node.rating)
                            .unwrap_or_default();

                        ui.horizontal(|ui| {
                            ui.label(rating.to_string());
                            let response = ui.selectable_label(is_selected, unit_name);
                            if response.clicked() {
                                data.selected_unit = Some(*unit_id);
                                Self::load_linked_data(&mut data, *unit_id);
                            }
                        });
                    }
                }
            });
            Ok(())
        })?;

        world.insert_resource(data);
        Ok(())
    }

    pub fn pane_current_representation(
        ui: &mut Ui,
        world: &mut World,
    ) -> Result<(), ExpressionError> {
        let data = world.resource::<ContentExplorerData>().clone();
        Self::generic_pane_current::<NUnitRepresentation>(ui, world, &data.linked_representations)
    }

    pub fn pane_representations_list(
        ui: &mut Ui,
        world: &mut World,
    ) -> Result<(), ExpressionError> {
        let data = world.resource::<ContentExplorerData>().clone();
        Self::generic_pane_list::<NUnitRepresentation>(
            ui,
            world,
            NodeKind::NUnitRepresentation,
            &data.linked_representations,
            data.owner_filter,
            data.selected_unit,
        )
    }

    pub fn pane_current_description(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        let data = world.resource::<ContentExplorerData>().clone();
        Self::generic_pane_current::<NUnitDescription>(ui, world, &data.linked_descriptions)
    }

    pub fn pane_descriptions_list(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        let data = world.resource::<ContentExplorerData>().clone();
        Self::generic_pane_list::<NUnitDescription>(
            ui,
            world,
            NodeKind::NUnitDescription,
            &data.linked_descriptions,
            data.owner_filter,
            data.selected_unit,
        )
    }

    pub fn pane_current_behavior(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        let data = world.resource::<ContentExplorerData>().clone();
        Self::generic_pane_current::<NUnitBehavior>(ui, world, &data.linked_behaviors)
    }

    pub fn pane_behaviors_list(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        let data = world.resource::<ContentExplorerData>().clone();
        Self::generic_pane_list::<NUnitBehavior>(
            ui,
            world,
            NodeKind::NUnitBehavior,
            &data.linked_behaviors,
            data.owner_filter,
            data.selected_unit,
        )
    }

    pub fn pane_current_stats(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        let data = world.resource::<ContentExplorerData>().clone();
        Self::generic_pane_current::<NUnitStats>(ui, world, &data.linked_stats)
    }

    pub fn pane_stats_list(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        let data = world.resource::<ContentExplorerData>().clone();
        Self::generic_pane_list::<NUnitStats>(
            ui,
            world,
            NodeKind::NUnitStats,
            &data.linked_stats,
            data.owner_filter,
            data.selected_unit,
        )
    }

    pub fn pane_houses_list(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        let data = world.resource::<ContentExplorerData>().clone();
        Self::generic_pane_list::<NHouse>(
            ui,
            world,
            NodeKind::NHouse,
            &data.linked_houses,
            data.owner_filter,
            data.selected_unit,
        )
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
