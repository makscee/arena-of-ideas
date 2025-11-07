use super::*;
use ron::ser::PrettyConfig;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Default, Clone)]
pub struct NodeAssetsManager {
    /// All nodes organized by kind and ID
    pub nodes: HashMap<NodeKind, HashMap<u64, NodeAsset>>,
    /// All links between nodes
    pub links: Vec<LinkAsset>,
    /// Base path for asset files
    pub assets_path: PathBuf,
}

impl NodeAssetsManager {
    /// Create a new NodeAssetsManager with the specified assets path
    pub fn new<P: AsRef<Path>>(assets_path: P) -> Self {
        Self {
            nodes: HashMap::new(),
            links: Vec::new(),
            assets_path: assets_path.as_ref().to_path_buf(),
        }
    }

    /// Add a node to the manager
    pub fn add_node(&mut self, kind: NodeKind, id: u64, data: String, owner_id: u64, rating: i32) {
        let asset = NodeAsset::new(data, owner_id, rating);

        self.nodes
            .entry(kind)
            .or_insert_with(HashMap::new)
            .insert(id, asset);
    }

    /// Add a node from a TNode database record
    pub fn add_node_from_tnode(&mut self, tnode: &TNode) {
        let kind = tnode.kind();
        self.add_node(
            kind,
            tnode.id,
            tnode.data.clone(),
            tnode.owner,
            tnode.rating,
        );
    }

    /// Add a link between two nodes
    pub fn add_link(
        &mut self,
        parent_id: u64,
        child_id: u64,
        parent_kind: String,
        child_kind: String,
        rating: i32,
        solid: bool,
    ) {
        let link = LinkAsset::new(parent_id, child_id, parent_kind, child_kind, rating, solid);
        self.links.push(link);
    }

    /// Add a link from a TNodeLink database record
    pub fn add_link_from_tnode_link(&mut self, link: &TNodeLink) {
        self.add_link(
            link.parent,
            link.child,
            link.parent_kind.clone(),
            link.child_kind.clone(),
            link.rating,
            link.solid,
        );
    }

    pub fn get_node(&self, kind: NodeKind, id: u64) -> Option<&NodeAsset> {
        self.nodes.get(&kind)?.get(&id)
    }

    pub fn get_nodes_of_kind(&self, kind: NodeKind) -> Option<&HashMap<u64, NodeAsset>> {
        self.nodes.get(&kind)
    }

    pub fn get_all_nodes(&self) -> &HashMap<NodeKind, HashMap<u64, NodeAsset>> {
        &self.nodes
    }

    pub fn get_links(&self) -> &Vec<LinkAsset> {
        &self.links
    }

    pub fn get_links_for_parent(&self, parent_id: u64) -> Vec<&LinkAsset> {
        self.links
            .iter()
            .filter(|link| link.0 == parent_id)
            .collect()
    }

    pub fn get_links_for_child(&self, child_id: u64) -> Vec<&LinkAsset> {
        self.links
            .iter()
            .filter(|link| link.1 == child_id)
            .collect()
    }

    pub fn get_links_by_kind(&self, parent_kind: &str, child_kind: &str) -> Vec<&LinkAsset> {
        self.links
            .iter()
            .filter(|link| link.2 == parent_kind && link.3 == child_kind)
            .collect()
    }

    /// Save all nodes and links to the file system
    ///
    /// Creates directory structure:
    /// - assets_path/{node_kind}/{node_id}.ron for each node
    /// - assets_path/links.ron for all links
    pub fn save_to_files(&self) -> Result<(), std::io::Error> {
        // Create the base assets directory
        fs::create_dir_all(&self.assets_path)?;

        // Save each node type to its own folder
        for (kind, nodes) in &self.nodes {
            let kind_dir = self.assets_path.join(kind.as_ref());
            fs::create_dir_all(&kind_dir)?;

            for (id, asset) in nodes {
                let file_path = kind_dir.join(format!("{}.ron", id));
                let content = ron::ser::to_string_pretty(asset, PrettyConfig::new())
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
                fs::write(file_path, content)?;
            }
        }

        // Save all links to a single file
        let links_path = self.assets_path.join("links.ron");
        let links_content = ron::ser::to_string_pretty(&self.links, PrettyConfig::new())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        fs::write(links_path, links_content)?;

        Ok(())
    }

    /// Load all nodes and links from the file system
    ///
    /// Reads from the directory structure created by save_to_files()
    pub fn load_from_files(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Clear existing data
        self.nodes.clear();
        self.links.clear();

        if !self.assets_path.exists() {
            return Ok(()); // No assets to load
        }

        // Load nodes by iterating through node kind directories
        for entry in fs::read_dir(&self.assets_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                let kind_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .ok_or("Invalid directory name")?;

                // Try to parse the directory name as a NodeKind
                if let Ok(kind) = NodeKind::from_str(kind_name) {
                    let mut kind_nodes = HashMap::new();

                    // Load all .ron files in this directory
                    for node_file in fs::read_dir(&path)? {
                        let node_file = node_file?;
                        let node_path = node_file.path();

                        if node_path.extension().and_then(|s| s.to_str()) == Some("ron") {
                            let file_stem = node_path
                                .file_stem()
                                .and_then(|s| s.to_str())
                                .ok_or("Invalid file name")?;

                            if let Ok(id) = file_stem.parse::<u64>() {
                                let content = fs::read_to_string(&node_path)?;
                                let asset: NodeAsset = ron::from_str(&content)?;
                                kind_nodes.insert(id, asset);
                            }
                        }
                    }

                    if !kind_nodes.is_empty() {
                        self.nodes.insert(kind, kind_nodes);
                    }
                }
            }
        }

        // Load links
        let links_path = self.assets_path.join("links.ron");
        if links_path.exists() {
            let links_content = fs::read_to_string(links_path)?;
            self.links = ron::from_str(&links_content)?;
        }

        Ok(())
    }

    pub fn export_node_as_string<T: NodeExt>(&self, kind: NodeKind, id: u64) -> Option<String> {
        let asset = self.get_node(kind, id)?;
        Some(asset.0.clone())
    }

    pub fn import_node_from_string<T: NodeExt + StringData>(
        &mut self,
        node: &T,
        owner_id: u64,
        rating: i32,
    ) {
        let kind = T::kind_s();
        let id = node.id();
        let data = node.get_data();
        self.add_node(kind, id, data, owner_id, rating);
    }

    pub fn get_node_count(&self) -> usize {
        self.nodes.values().map(|nodes| nodes.len()).sum()
    }

    pub fn get_link_count(&self) -> usize {
        self.links.len()
    }

    pub fn get_kinds(&self) -> Vec<NodeKind> {
        self.nodes.keys().cloned().collect()
    }

    pub fn remove_node(&mut self, kind: NodeKind, id: u64) -> Option<NodeAsset> {
        // Remove the node
        let removed_node = self.nodes.get_mut(&kind)?.remove(&id);

        // Remove associated links
        self.links.retain(|link| link.0 != id && link.1 != id);

        removed_node
    }

    pub fn remove_link(&mut self, parent_id: u64, child_id: u64) -> bool {
        let initial_len = self.links.len();
        self.links
            .retain(|link| !(link.0 == parent_id && link.1 == child_id));
        self.links.len() < initial_len
    }

    pub fn update_node_rating(&mut self, kind: NodeKind, id: u64, new_rating: i32) -> bool {
        if let Some(asset) = self
            .nodes
            .get_mut(&kind)
            .and_then(|nodes| nodes.get_mut(&id))
        {
            asset.2 = new_rating;
            true
        } else {
            false
        }
    }

    pub fn update_link_rating(&mut self, parent_id: u64, child_id: u64, new_rating: i32) -> bool {
        for link in &mut self.links {
            if link.0 == parent_id && link.1 == child_id {
                link.4 = new_rating;
                return true;
            }
        }
        false
    }

    pub fn get_nodes_by_owner(&self, owner_id: u64) -> HashMap<NodeKind, HashMap<u64, &NodeAsset>> {
        let mut result = HashMap::new();

        for (kind, nodes) in &self.nodes {
            let mut owner_nodes = HashMap::new();
            for (id, asset) in nodes {
                if asset.1 == owner_id {
                    owner_nodes.insert(*id, asset);
                }
            }
            if !owner_nodes.is_empty() {
                result.insert(*kind, owner_nodes);
            }
        }

        result
    }

    pub fn get_top_rated_nodes(&self, kind: NodeKind, limit: usize) -> Vec<(u64, &NodeAsset)> {
        if let Some(nodes) = self.nodes.get(&kind) {
            let mut sorted_nodes: Vec<_> = nodes.iter().collect();
            sorted_nodes.sort_by(|a, b| b.1.2.cmp(&a.1.2));
            sorted_nodes
                .into_iter()
                .take(limit)
                .map(|(id, asset)| (*id, asset))
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
        self.links.clear();
    }

    pub fn merge(&mut self, other: NodeAssetsManager) {
        // Merge nodes
        for (kind, nodes) in other.nodes {
            let entry = self.nodes.entry(kind).or_insert_with(HashMap::new);
            for (id, asset) in nodes {
                entry.insert(id, asset);
            }
        }

        // Merge links (avoid duplicates)
        for link in other.links {
            if !self
                .links
                .iter()
                .any(|existing_link| existing_link.0 == link.0 && existing_link.1 == link.1)
            {
                self.links.push(link);
            }
        }
    }
}

// Convenience functions for global usage

/// Get the default world repository path
/// Checks for AOI_WORLD_PATH environment variable, falls back to default
pub fn get_world_assets_path() -> PathBuf {
    if let Ok(env_path) = std::env::var("AOI_WORLD_PATH") {
        PathBuf::from(env_path).join("assets/nodes")
    } else {
        PathBuf::from("../arena-of-ideas-world/assets/nodes")
    }
}

/// Load node assets from the arena-of-ideas-world repository directory
pub fn load_node_assets() -> Result<NodeAssetsManager, Box<dyn std::error::Error>> {
    let mut manager = NodeAssetsManager::new(get_world_assets_path());
    manager.load_from_files()?;
    Ok(manager)
}

/// Load node assets from a custom path
pub fn load_node_assets_from_path<P: AsRef<Path>>(
    path: P,
) -> Result<NodeAssetsManager, Box<dyn std::error::Error>> {
    let mut manager = NodeAssetsManager::new(path);
    manager.load_from_files()?;
    Ok(manager)
}

/// Save node assets to files using the manager's configured path
pub fn save_node_assets(manager: &NodeAssetsManager) -> Result<(), std::io::Error> {
    manager.save_to_files()
}

/// Create a new NodeAssetsManager with the default world repository path
pub fn create_world_assets_manager() -> NodeAssetsManager {
    NodeAssetsManager::new(get_world_assets_path())
}
