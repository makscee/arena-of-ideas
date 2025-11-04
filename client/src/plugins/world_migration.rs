use super::*;
use std::path::Path;

pub struct WorldMigrationPlugin;

impl Plugin for WorldMigrationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::WorldDownload), world_download_system)
            .add_systems(OnEnter(GameState::WorldUpload), world_upload_system);
    }
}

fn world_download_system(world: &mut World) {
    info!("Starting world assets download...");

    let world_path = get_world_assets_path();

    if let Err(e) = clear_world_assets_folder(&world_path) {
        format!("Failed to clear world assets folder: {}", e).notify_error(world);
        app_exit(world);
        return;
    }

    match download_world_assets() {
        Ok(count) => {
            format!(
                "Successfully downloaded {} nodes and links to world repository",
                count
            )
            .notify(world);
            info!("World download completed successfully");
        }
        Err(e) => {
            format!("Failed to download world assets: {}", e).notify_error(world);
        }
    }

    app_exit(world);
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

fn download_world_assets() -> Result<usize, Box<dyn std::error::Error>> {
    download_world_assets_to_path(&get_world_assets_path())
}

pub fn download_world_assets_to_path<P: AsRef<Path>>(
    path: P,
) -> Result<usize, Box<dyn std::error::Error>> {
    let path = path.as_ref();

    // Clear existing assets folder
    clear_world_assets_folder(path)?;

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

    manager.save_to_files()?;

    Ok(total_count)
}

fn upload_world_assets() -> Result<usize, Box<dyn std::error::Error>> {
    upload_world_assets_from_path(&get_world_assets_path())
}

pub fn upload_world_assets_from_path<P: AsRef<Path>>(
    path: P,
) -> Result<usize, Box<dyn std::error::Error>> {
    let manager = load_node_assets_from_path(path)?;
    dbg!(&manager);

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
