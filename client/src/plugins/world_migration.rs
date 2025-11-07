use super::*;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter, Display, AsRefStr)]
enum OwnerFilter {
    Players,
    OwnedByPlayers,
    Arena,
    Zero,
    Core,
}

impl OwnerFilter {
    fn owner_id(&self) -> u64 {
        match self {
            OwnerFilter::Players => ID_PLAYERS,
            OwnerFilter::OwnedByPlayers => ID_PLAYERS,
            OwnerFilter::Arena => ID_ARENA,
            OwnerFilter::Zero => 0,
            OwnerFilter::Core => ID_CORE,
        }
    }

    fn matches_owner(&self, owner_id: u64, _world: &World) -> bool {
        match self {
            OwnerFilter::OwnedByPlayers => cn()
                .db
                .nodes_world()
                .iter()
                .find(|n| n.id == owner_id)
                .map(|n| {
                    if let Ok(node) = n.to_node::<NPlayer>() {
                        node.kind() == NodeKind::NPlayer
                    } else {
                        false
                    }
                })
                .unwrap_or(false),
            _ => owner_id == self.owner_id(),
        }
    }
}

#[derive(Resource)]
struct WorldDownloadState {
    original_manager: Option<NodeAssetsManager>,
    filtered_manager: Option<NodeAssetsManager>,
    owner_filters: HashSet<OwnerFilter>,
    kind_filters: HashSet<NodeKind>,
    node_id_filters: HashSet<(NodeKind, u64)>,
    selected_kind: NodeKind,
}

impl Default for WorldDownloadState {
    fn default() -> Self {
        Self {
            original_manager: None,
            filtered_manager: None,
            owner_filters: [OwnerFilter::Zero, OwnerFilter::Core, OwnerFilter::Arena]
                .iter()
                .copied()
                .collect(),
            kind_filters: HashSet::new(),
            node_id_filters: HashSet::new(),
            selected_kind: NodeKind::None,
        }
    }
}

pub struct WorldMigrationPlugin;

impl Plugin for WorldMigrationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldDownloadState>()
            .add_systems(OnEnter(GameState::WorldDownload), world_download_init)
            .add_systems(OnEnter(GameState::WorldUpload), world_upload_system);
    }
}

fn world_download_init(mut state: ResMut<WorldDownloadState>) {
    info!("Initializing world download...");

    match download_and_store_world_assets() {
        Ok(manager) => {
            state.original_manager = Some(manager.clone());
            state.filtered_manager = Some(manager);
            state.kind_filters = default();
            state.node_id_filters = HashSet::new();
            apply_filters(&mut state);
        }
        Err(e) => {
            op(move |world| {
                format!("Failed to download world assets: {}", e).notify_error(world);
                app_exit(world);
            });
        }
    }
}

pub fn world_download_ui_system(ui: &mut Ui, world: &mut World) {
    let mut state = world.resource_mut::<WorldDownloadState>();
    let mut owner_filters = state.owner_filters.clone();
    let mut kind_filters = state.kind_filters.clone();
    let mut node_id_filters = state.node_id_filters.clone();
    let mut selected_kind = state.selected_kind;
    let filtered_manager = state.filtered_manager.clone();
    let original_manager = state.original_manager.clone();

    let mut filters_changed = false;
    ui.heading("World Assets Download");

    ui.separator();
    ui.label("Owner Filters:");
    ui.horizontal(|ui| {
        let mut to_remove = None;
        let mut filters_list: Vec<_> = owner_filters.iter().copied().collect();
        filters_list.sort_by_key(|f| format!("{:?}", f));

        for (_idx, filter) in filters_list.iter().enumerate() {
            ui.horizontal(|ui| {
                ui.label(filter.to_string());
                if ui.button("x").clicked() {
                    to_remove = Some(*filter);
                    filters_changed = true;
                }
            });
        }

        if let Some(filter) = to_remove {
            owner_filters.remove(&filter);
        }
    });

    ui.horizontal(|ui| {
        ui.label("Add Owner Filter:");
        for variant in OwnerFilter::iter() {
            if !owner_filters.contains(&variant) && ui.button(variant.to_string()).clicked() {
                owner_filters.insert(variant);
                filters_changed = true;
            }
        }
    });

    ui.separator();
    ui.label("Node Kind Filters:");
    ui.horizontal(|ui| {
        let mut to_remove = None;
        let mut filters_list: Vec<_> = kind_filters.iter().copied().collect();
        filters_list.sort_by_key(|kind| *kind);

        for (_idx, kind) in filters_list.iter().enumerate() {
            ui.horizontal(|ui| {
                ui.label(format!("{:?}", kind));
                if ui.button("x").clicked() {
                    to_remove = Some(*kind);
                    filters_changed = true;
                }
            });
        }

        if let Some(kind) = to_remove {
            kind_filters.remove(&kind);
        }
    });

    ui.horizontal(|ui| {
        ui.label("Add Kind Filter:");
        Selector::ui_enum(&mut selected_kind, ui);

        if ui.button("Add").clicked() {
            if selected_kind != NodeKind::None {
                kind_filters.insert(selected_kind);
                filters_changed = true;
            }
        }
    });

    ui.separator();
    ui.label("Excluded Nodes Preview:");

    if let Some(ref original) = original_manager {
        let excluded_nodes: Vec<_> = original
            .get_all_nodes()
            .iter()
            .flat_map(|(kind, nodes)| {
                nodes
                    .iter()
                    .filter_map(|(id, asset)| {
                        if node_id_filters.contains(&(*kind, *id)) {
                            Some((*kind, *id, asset.owner_id()))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        if excluded_nodes.is_empty() {
            ui.label("No excluded nodes");
        } else {
            ui.label(format!("Total Excluded: {}", excluded_nodes.len()));
            ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                for (kind, id, owner) in excluded_nodes {
                    ui.horizontal(|ui| {
                        if ui.button("Remove exclusion").clicked() {
                            node_id_filters.remove(&(kind, id));
                            filters_changed = true;
                        }
                        ui.label(format!("{:?}", kind));
                        ui.label(format!("ID: {}", id));
                        ui.label(format!("Owner: {}", owner));
                        ui.label(original.get_node(kind, id).unwrap().data());
                    });
                }
            });
        }
    }

    ui.separator();
    ui.label("Preview:");

    if let Some(ref manager) = filtered_manager {
        let mut total_nodes = 0;

        let nodes = manager
            .get_all_nodes()
            .iter()
            .sorted_by_key(|(kind, _)| **kind);

        for (kind, nodes) in nodes {
            total_nodes += nodes.len();
            CollapsingHeader::new(format!("{:?} ({})", kind, nodes.len()))
                .default_open(false)
                .id_salt(kind)
                .show(ui, |ui| {
                    for (id, asset) in nodes {
                        ui.horizontal(|ui| {
                            let is_excluded = node_id_filters.contains(&(*kind, *id));
                            let mut excluded = is_excluded;

                            if ui.checkbox(&mut excluded, "").changed() {
                                if excluded {
                                    node_id_filters.insert((*kind, *id));
                                    if let Some(ref orig) = state.original_manager.clone() {
                                        exclude_node_recursive(
                                            &mut node_id_filters,
                                            orig,
                                            *kind,
                                            *id,
                                        );
                                    }
                                } else {
                                    node_id_filters.remove(&(*kind, *id));
                                }
                                filters_changed = true;
                            }

                            ui.label(format!("ID: {}", id));
                            ui.label(format!("Owner: {}", asset.owner_id()));
                            ui.label(asset.data());
                        });
                    }
                });
        }
        format!("Total Nodes: {total_nodes}").label(ui);
        ui.separator();
        ui.label("Links:");
        let links = manager.get_links();
        ui.label(format!("Total Links: {}", links.len()));
        for link in links.iter().take(50) {
            ui.label(format!(
                "Parent: {} ({}), Child: {} ({})",
                link.0, link.2, link.1, link.3
            ));
        }
        if links.len() > 50 {
            ui.label(format!("... and {} more links", links.len() - 50));
        }
    }

    ui.separator();
    ui.horizontal(|ui| {
        if ui.button("Save").clicked() {
            if let Some(manager) = state.filtered_manager.clone() {
                let result = save_filtered_assets(&manager);
                match result {
                    Ok(count) => {
                        op(move |world| {
                            format!(
                                "Successfully saved {} nodes and links to world repository",
                                count
                            )
                            .notify(world);
                            info!("World download completed successfully");
                            app_exit(world);
                        });
                    }
                    Err(e) => {
                        op(move |world| {
                            format!("Failed to save assets: {}", e).notify_error(world);
                        });
                    }
                }
            }
        }

        if ui.button("Exit").clicked() {
            op(|world| {
                app_exit(world);
            });
        }
    });

    state.owner_filters = owner_filters;
    state.kind_filters = kind_filters;
    state.node_id_filters = node_id_filters;
    state.selected_kind = selected_kind;

    if filters_changed {
        apply_filters(&mut state);
    }
}

fn apply_filters(state: &mut WorldDownloadState) {
    if let Some(ref original) = state.original_manager {
        let mut filtered = NodeAssetsManager::new(original.assets_path.clone());

        for (kind, nodes) in original.get_all_nodes() {
            for (id, asset) in nodes {
                if state.node_id_filters.contains(&(*kind, *id)) {
                    continue;
                }

                let owner_matches = state
                    .owner_filters
                    .iter()
                    .any(|f| f.matches_owner(asset.owner_id(), &World::new()));

                if !owner_matches {
                    continue;
                }

                if state.kind_filters.contains(kind) {
                    continue;
                }

                filtered.add_node(
                    *kind,
                    *id,
                    asset.data().clone(),
                    asset.owner_id(),
                    asset.rating(),
                );
            }
        }

        let filtered_node_set: HashSet<(NodeKind, u64)> = filtered
            .get_all_nodes()
            .iter()
            .flat_map(|(kind, nodes)| nodes.keys().map(move |id| (*kind, *id)))
            .collect();

        for link in original.get_links() {
            let parent_kind = NodeKind::from_str(&link.2).unwrap_or_default();
            let child_kind = NodeKind::from_str(&link.3).unwrap_or_default();

            let parent_in_filter = filtered_node_set.contains(&(parent_kind, link.0));
            let child_in_filter = filtered_node_set.contains(&(child_kind, link.1));

            if parent_in_filter && child_in_filter {
                filtered.add_link(
                    link.0,
                    link.1,
                    link.2.clone(),
                    link.3.clone(),
                    link.4,
                    link.5 != 0,
                );
            }
        }

        state.filtered_manager = Some(filtered);
    }
}

fn exclude_node_recursive(
    node_id_filters: &mut HashSet<(NodeKind, u64)>,
    original: &NodeAssetsManager,
    _kind: NodeKind,
    node_id: u64,
) {
    let children = original.get_links_for_parent(node_id);
    let children_to_process: Vec<_> = children
        .iter()
        .map(|link| {
            (
                NodeKind::from_str(&link.3).unwrap_or(NodeKind::None),
                link.1,
            )
        })
        .collect();

    for (child_kind, child_id) in children_to_process {
        node_id_filters.insert((child_kind, child_id));
        exclude_node_recursive(node_id_filters, original, child_kind, child_id);
    }
}

fn download_and_store_world_assets() -> Result<NodeAssetsManager, String> {
    let mut manager = NodeAssetsManager::new(std::env::temp_dir());

    for node in cn().db.nodes_world().iter() {
        manager.add_node_from_tnode(&node);
    }

    for link in cn().db.node_links().iter() {
        manager.add_link_from_tnode_link(&link);
    }

    Ok(manager)
}

fn save_filtered_assets(manager: &NodeAssetsManager) -> Result<usize, String> {
    let world_path = get_world_assets_path();

    clear_world_assets_folder(&world_path).map_err(|e| e.to_string())?;

    let mut final_manager = NodeAssetsManager::new(&world_path);

    for (kind, nodes) in manager.get_all_nodes() {
        for (id, asset) in nodes {
            final_manager.add_node(
                *kind,
                *id,
                asset.data().clone(),
                asset.owner_id(),
                asset.rating(),
            );
        }
    }

    for link in manager
        .get_links()
        .into_iter()
        .sorted_by_key(|l| format!("{}_{}", l.0, l.1))
    {
        final_manager.add_link(
            link.0,
            link.1,
            link.2.clone(),
            link.3.clone(),
            link.4,
            link.5 != 0,
        );
    }

    let total_count = final_manager
        .get_all_nodes()
        .iter()
        .map(|(_, n)| n.len())
        .sum::<usize>()
        + final_manager.get_links().len();

    final_manager.save_to_files().map_err(|e| e.to_string())?;

    Ok(total_count)
}

fn clear_world_assets_folder<P: AsRef<Path>>(path: P) -> Result<(), std::io::Error> {
    let path = path.as_ref();

    if !path.exists() {
        return Ok(());
    }

    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();

        if entry_path.is_dir() {
            std::fs::remove_dir_all(&entry_path)?;
        } else {
            std::fs::remove_file(&entry_path)?;
        }
    }

    Ok(())
}

pub fn download_world_assets_to_path<P: AsRef<Path>>(path: P) -> Result<usize, String> {
    let path = path.as_ref();

    clear_world_assets_folder(path).map_err(|e| e.to_string())?;

    let mut manager = NodeAssetsManager::new(path);

    let mut total_count = 0;

    for node in cn().db.nodes_world().iter() {
        manager.add_node_from_tnode(&node);
        total_count += 1;
    }

    for link in cn().db.node_links().iter() {
        manager.add_link_from_tnode_link(&link);
        total_count += 1;
    }

    manager.save_to_files().map_err(|e| e.to_string())?;

    Ok(total_count)
}

fn world_upload_system(world: &mut World) {
    info!("Starting world assets upload...");

    match upload_world_assets() {
        Ok(count) => {
            format!(
                "Successfully uploaded {} nodes and links from world repository",
                count
            )
            .notify(world);
            info!("World upload completed successfully");
        }
        Err(e) => {
            format!("Failed to upload world assets: {}", e).notify_error(world);
        }
    }

    app_exit(world);
}

fn upload_world_assets() -> Result<usize, String> {
    upload_world_assets_from_path(&get_world_assets_path())
}

pub fn upload_world_assets_from_path<P: AsRef<Path>>(path: P) -> Result<usize, String> {
    let manager = load_node_assets_from_path(path).map_err(|e| e.to_string())?;

    let mut all_nodes: Vec<(u64, String, NodeAsset)> = default();
    let mut all_links: Vec<LinkAsset> = default();

    for (kind, nodes) in manager.nodes {
        let kind = kind.to_string();
        for (id, node) in nodes {
            all_nodes.push((id, kind.clone(), node));
        }
    }

    for link in manager.links {
        all_links.push(link);
    }

    let total_count = all_nodes.len() + all_links.len();

    let all_nodes = all_nodes
        .into_iter()
        .map(|n| ron::to_string(&n).unwrap())
        .collect_vec();
    let all_links = all_links
        .into_iter()
        .map(|l| ron::to_string(&l).unwrap())
        .collect_vec();

    cn().reducers
        .admin_upload_world(global_settings().clone(), all_nodes, all_links)
        .unwrap();

    Ok(total_count)
}
