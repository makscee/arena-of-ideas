use super::node_maps::*;

use crate::prelude::*;
use crate::resources::game_option::player_id;
use crate::stdb::{RemoteTables, TNodeLink};
use once_cell::sync::Lazy;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

// Node update actions for syncing sources
#[derive(Debug, Clone)]
pub enum NodeUpdateAction {
    Insert {
        id: u64,
        owner: u64,
        kind: NodeKind,
        data: String,
    },
    Update {
        id: u64,
        data: String,
    },
    Delete {
        id: u64,
    },
    LinkAdded {
        parent: u64,
        child: u64,
        link: TNodeLink,
    },
    LinkRemoved {
        parent: u64,
        child: u64,
    },
    LinkSelectionChanged {
        player_id: u64,
        parent_id: u64,
        selected_link_id: u64,
    },
}

#[derive(Clone)]
pub enum LinkStrategy {
    /// Only solid links
    Solid,
    /// Highest rated links first
    TopRated,
    /// Player-selected links only
    Selected,
    /// Custom filtering function
    Custom(Arc<dyn Fn(&[TNodeLink]) -> Vec<TNodeLink> + Send + Sync>),
}

impl std::fmt::Debug for LinkStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Solid => write!(f, "Solid"),
            Self::TopRated => write!(f, "TopRated"),
            Self::Selected => write!(f, "Selected"),
            Self::Custom(_) => write!(f, "Custom"),
        }
    }
}

// Individual node source with its own World and link strategy
#[derive(Clone)]
pub struct NodeSource {
    world: Arc<RwLock<World>>,
    strategy: LinkStrategy,
    node_map: Arc<RwLock<SmartNodeMap>>,
    links: Arc<RwLock<NodeLinks>>,
}

impl NodeSource {
    pub fn new(strategy: LinkStrategy) -> Self {
        let mut world = World::new();
        world.init_resource::<SmartNodeMap>();
        world.init_resource::<NodeLinks>();

        Self {
            world: Arc::new(RwLock::new(world)),
            strategy,
            node_map: Arc::new(RwLock::new(SmartNodeMap::new())),
            links: Arc::new(RwLock::new(NodeLinks::new())),
        }
    }

    pub fn from_world(world: World, strategy: LinkStrategy) -> Self {
        Self {
            world: Arc::new(RwLock::new(world)),
            strategy,
            node_map: Arc::new(RwLock::new(SmartNodeMap::new())),
            links: Arc::new(RwLock::new(NodeLinks::new())),
        }
    }

    pub fn world(&self) -> Arc<RwLock<World>> {
        self.world.clone()
    }

    pub fn node_map(&self) -> Arc<RwLock<SmartNodeMap>> {
        self.node_map.clone()
    }

    pub fn links(&self) -> Arc<RwLock<NodeLinks>> {
        self.links.clone()
    }

    pub fn update(&mut self, action: &NodeUpdateAction, db: &RemoteTables) {
        match action {
            NodeUpdateAction::Insert {
                id,
                owner,
                kind,
                data,
            } => {
                self.insert_node(*id, *owner, kind.clone(), data.clone());
            }
            NodeUpdateAction::Update { id, data } => {
                self.update_node(*id, data.clone());
            }
            NodeUpdateAction::Delete { id } => {
                self.delete_node(*id);
            }
            NodeUpdateAction::LinkAdded {
                parent,
                child,
                link,
            } => {
                self.add_link_from_db(*parent, *child, link, db);
            }
            NodeUpdateAction::LinkRemoved { parent, child } => {
                self.remove_link(*parent, *child);
            }
            NodeUpdateAction::LinkSelectionChanged { .. } => {
                if matches!(self.strategy, LinkStrategy::Selected) {
                    self.rebuild_from_db(db);
                }
            }
        }
    }

    pub fn rebuild_from_db(&mut self, db: &RemoteTables) {
        let mut world = self.world.write().unwrap();
        let mut node_map = self.node_map.write().unwrap();
        let mut links = self.links.write().unwrap();

        // Clear existing data
        world.clear_all();
        node_map.clear();
        links.clear();

        // Rebuild nodes using smart insertion
        for node in db.nodes_world().iter() {
            let kind = node.kind.parse().unwrap_or(NodeKind::NArena);
            let entity = node_map.insert_smart(&mut world, node.id, node.owner, kind.clone());

            // Spawn the actual node component
            self.spawn_node_component(&mut world, entity, node.id, node.owner, kind, &node.data);
        }

        // Rebuild links based on strategy
        let all_links: Vec<TNodeLink> = db.node_links().iter().collect();
        let filtered_links = self.filter_links(&all_links, db);

        for link in filtered_links {
            self.add_link_internal(&mut world, &node_map, &mut links, link);
        }
    }

    fn spawn_node_component(
        &self,
        world: &mut World,
        entity: Entity,
        id: u64,
        owner: u64,
        kind: NodeKind,
        data: &str,
    ) {
        // Use the node system to spawn the appropriate component
        // TODO: Replace with proper node_kind_match macro or implement per-kind logic
        // node_kind_match!(kind, {
        //     let mut node = NodeType::default();
        //     if node.inject_data(data).is_ok() {
        //         node.set_id(id);
        //         node.set_owner(owner);
        //         world.entity_mut(entity).insert(node);
        //     }
        // });

        // Placeholder: Add basic components for now
        world
            .entity_mut(entity)
            .insert((NodeKindMarker(kind), NodeId(id), NodeOwner(owner)));
    }

    fn filter_links(&self, links: &[TNodeLink], db: &RemoteTables) -> Vec<TNodeLink> {
        match &self.strategy {
            LinkStrategy::Solid => links.iter().filter(|l| l.solid).cloned().collect(),
            LinkStrategy::TopRated => {
                let mut by_parent: HashMap<u64, Vec<TNodeLink>> = HashMap::new();
                for link in links {
                    by_parent.entry(link.parent).or_default().push(link.clone());
                }

                let mut result = Vec::new();
                for (_, mut parent_links) in by_parent {
                    parent_links.sort_by(|a, b| {
                        b.rating
                            .cmp(&a.rating)
                            .then(b.solid.cmp(&a.solid))
                            .then(a.id.cmp(&b.id))
                    });

                    // Take top rated link for each child kind
                    let mut seen_kinds = HashSet::new();
                    for link in parent_links {
                        if seen_kinds.insert(link.child_kind.clone()) {
                            result.push(link);
                        }
                    }
                }
                result
            }
            LinkStrategy::Selected => {
                let player_id = player_id();
                let selections: HashSet<u64> = db
                    .player_link_selections()
                    .iter()
                    .filter(|s| s.player_id == player_id)
                    .map(|s| s.selected_link_id)
                    .collect();

                links
                    .iter()
                    .filter(|l| selections.contains(&l.id))
                    .cloned()
                    .collect()
            }
            LinkStrategy::Custom(filter) => filter(links),
        }
    }

    pub fn insert_node(&mut self, id: u64, owner: u64, kind: NodeKind, data: String) {
        let mut world = self.world.write().unwrap();
        let mut node_map = self.node_map.write().unwrap();

        let entity = node_map.insert_smart(&mut world, id, owner, kind.clone());
        self.spawn_node_component(&mut world, entity, id, owner, kind, &data);
    }

    fn update_node(&mut self, id: u64, data: String) {
        let world = self.world.read().unwrap();
        let node_map = self.node_map.read().unwrap();

        if let Some(entity) = node_map.get_entity(id) {
            if let Some(kind) = node_map.get_node_kind(id) {
                drop(world);
                let mut world = self.world.write().unwrap();

                // Update the node component with new data
                // TODO: Replace with proper node_kind_match macro or implement per-kind logic
                // node_kind_match!(kind, {
                //     if let Some(mut node) = world.get_mut::<NodeType>(entity) {
                //         let _ = node.inject_data(&data);
                //     }
                // });

                // Placeholder: For now just update the basic components if needed
                // Individual node component updates would go here based on kind
            }
        }
    }

    pub fn delete_node(&mut self, id: u64) {
        let mut world = self.world.write().unwrap();
        let mut node_map = self.node_map.write().unwrap();
        let mut links = self.links.write().unwrap();

        // Remove all links first
        links.remove_all_links(&mut world, id, &node_map);

        // Remove node (this will despawn entity if it's the last component)
        node_map.remove_smart(&mut world, id);
    }

    fn add_link_from_db(&mut self, parent: u64, child: u64, link: &TNodeLink, db: &RemoteTables) {
        let filtered = self.filter_links(&[link.clone()], db);
        if !filtered.is_empty() {
            let mut world = self.world.write().unwrap();
            let node_map = self.node_map.read().unwrap();
            let mut links = self.links.write().unwrap();
            self.add_link_internal(&mut world, &node_map, &mut links, link.clone());
        }
    }

    fn add_link_internal(
        &self,
        world: &mut World,
        node_map: &SmartNodeMap,
        links: &mut NodeLinks,
        link: TNodeLink,
    ) {
        // Determine link type based on the relationship in raw_nodes.rs
        let parent_kind = node_map.get_node_kind(link.parent);
        let child_kind: Option<NodeKind> = link.child_kind.parse().ok();

        if let (Some(parent_kind), Some(child_kind)) = (parent_kind, child_kind) {
            // Check if this is an owned or reference relationship
            // TODO: Implement has_owned_child() method in NodeKind
            if false {
                // parent_kind.has_owned_child(&child_kind) {
                links.add_owned_link(world, link.parent, link.child, node_map);
            // TODO: Implement has_ref_child() method in NodeKind
            } else if false {
                // parent_kind.has_ref_child(&child_kind) {
                links.add_ref_link(world, link.parent, link.child, node_map);
            }
        }
    }

    fn remove_link(&mut self, parent: u64, child: u64) {
        let mut world = self.world.write().unwrap();
        let node_map = self.node_map.read().unwrap();
        let mut links = self.links.write().unwrap();

        // Remove both types of links (will be no-op if not present)
        links.remove_owned_link(&mut world, parent, child, &node_map);
        links.remove_ref_link(&mut world, parent, child, &node_map);
    }
}

// Global static node sources
pub struct NodeSources {
    solid: Arc<RwLock<NodeSource>>,
    top: Arc<RwLock<NodeSource>>,
    selected: Arc<RwLock<NodeSource>>,
    custom: Arc<RwLock<HashMap<String, NodeSource>>>,
}

impl NodeSources {
    fn new() -> Self {
        Self {
            solid: Arc::new(RwLock::new(NodeSource::new(LinkStrategy::Solid))),
            top: Arc::new(RwLock::new(NodeSource::new(LinkStrategy::TopRated))),
            selected: Arc::new(RwLock::new(NodeSource::new(LinkStrategy::Selected))),
            custom: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn solid(&self) -> Arc<RwLock<NodeSource>> {
        self.solid.clone()
    }

    pub fn top(&self) -> Arc<RwLock<NodeSource>> {
        self.top.clone()
    }

    pub fn selected(&self) -> Arc<RwLock<NodeSource>> {
        self.selected.clone()
    }

    pub fn get_or_create_custom(
        &self,
        key: String,
        strategy: LinkStrategy,
    ) -> Arc<RwLock<NodeSource>> {
        let mut custom = self.custom.write().unwrap();
        if !custom.contains_key(&key) {
            custom.insert(key.clone(), NodeSource::new(strategy));
        }
        Arc::new(RwLock::new(custom.get(&key).unwrap().clone()))
    }

    pub fn update_all(&self, action: &NodeUpdateAction, db: &RemoteTables) {
        self.solid.write().unwrap().update(action, db);
        self.top.write().unwrap().update(action, db);
        self.selected.write().unwrap().update(action, db);

        let mut custom = self.custom.write().unwrap();
        for source in custom.values_mut() {
            source.update(action, db);
        }
    }
}

// Global static instance
static NODE_SOURCES: Lazy<NodeSources> = Lazy::new(NodeSources::new);

pub fn node_sources() -> &'static NodeSources {
    &NODE_SOURCES
}
