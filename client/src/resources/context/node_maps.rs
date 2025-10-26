use super::relationships::*;
use crate::prelude::*;
use std::collections::{HashMap, HashSet};

// Smart node entity mapping that handles component grouping and relationships
#[derive(Resource, Default, Clone)]
pub struct SmartNodeMap {
    // Maps node ID to entity
    node_to_entity: HashMap<u64, Entity>,
    // Maps entity to set of node IDs it contains
    entity_to_nodes: HashMap<Entity, HashSet<u64>>,
    // Maps node ID to its kind
    node_kinds: HashMap<u64, NodeKind>,
    // Node relationship tracking
    links: NodeLinks,
}

impl SmartNodeMap {
    pub fn new() -> Self {
        Self {
            links: NodeLinks::new(),
            ..Default::default()
        }
    }

    /// Insert or update a node, intelligently deciding which entity to use
    pub fn insert_smart(
        &mut self,
        world: &mut World,
        id: u64,
        owner: u64,
        kind: NodeKind,
    ) -> Entity {
        // Check if this node already exists
        if let Some(existing_entity) = self.node_to_entity.get(&id) {
            return *existing_entity;
        }

        // Check if we should attach to an existing entity based on component relationships
        let target_entity = self.find_target_entity(id, &kind);

        let entity = match target_entity {
            Some(entity) => {
                // Attach to existing entity
                self.attach_to_entity(world, entity, id, kind.clone());
                entity
            }
            None => {
                // Create new entity
                let entity = world
                    .spawn((NodeKindMarker(kind.clone()), NodeId(id), NodeOwner(owner)))
                    .id();
                entity
            }
        };

        // Update mappings
        self.node_to_entity.insert(id, entity);
        self.entity_to_nodes.entry(entity).or_default().insert(id);
        self.node_kinds.insert(id, kind);

        entity
    }

    /// Find the appropriate entity to attach a component to
    fn find_target_entity(&self, node_id: u64, kind: &NodeKind) -> Option<Entity> {
        // Check component_parent relationships - if this node should be a component of a parent
        if let Some(parent_kind) = kind.component_parent() {
            // Look for existing links where we are the child of a parent_kind
            for (&existing_id, &existing_entity) in &self.node_to_entity {
                if let Some(existing_kind) = self.node_kinds.get(&existing_id) {
                    if *existing_kind == parent_kind {
                        // Check if there's a link from existing_id to node_id (parent -> child)
                        if self.links.has_owned_link(existing_id, node_id)
                            || self.links.has_ref_link(existing_id, node_id)
                        {
                            return Some(existing_entity);
                        }
                    }
                }
            }
        }

        // Check component_children relationships - if this node has component children
        for child_kind in kind.component_children() {
            // Look for existing links where we are the parent of a child_kind
            for (&existing_id, &existing_entity) in &self.node_to_entity {
                if let Some(existing_kind) = self.node_kinds.get(&existing_id) {
                    if *existing_kind == child_kind {
                        // Check if there's a link from node_id to existing_id (parent -> child)
                        if self.links.has_owned_link(node_id, existing_id)
                            || self.links.has_ref_link(node_id, existing_id)
                        {
                            return Some(existing_entity);
                        }
                    }
                }
            }
        }

        None
    }

    fn attach_to_entity(&mut self, world: &mut World, entity: Entity, id: u64, kind: NodeKind) {
        // Add the node markers to existing entity
        world
            .entity_mut(entity)
            .insert((NodeId(id), NodeKindMarker(kind)));
    }

    /// Remove a node, only despawning entity if it's the last component
    pub fn remove_smart(&mut self, world: &mut World, id: u64) -> bool {
        if let Some(entity) = self.node_to_entity.remove(&id) {
            // Remove all links for this node
            self.remove_all_links_for_node(world, id);

            // Remove from entity's node set
            if let Some(node_set) = self.entity_to_nodes.get_mut(&entity) {
                node_set.remove(&id);

                // If this was the last node on the entity, despawn it
                if node_set.is_empty() {
                    self.entity_to_nodes.remove(&entity);
                    world.despawn(entity);
                    return true;
                }
            }

            self.node_kinds.remove(&id);
        }
        false
    }

    pub fn get_entity(&self, id: u64) -> Option<Entity> {
        self.node_to_entity.get(&id).copied()
    }

    pub fn get_node_ids(&self, entity: Entity) -> Vec<u64> {
        self.entity_to_nodes
            .get(&entity)
            .map(|set| set.iter().copied().collect())
            .unwrap_or_default()
    }

    pub fn get_node_kind(&self, id: u64) -> Option<NodeKind> {
        self.node_kinds.get(&id).cloned()
    }

    pub fn clear(&mut self) {
        self.node_to_entity.clear();
        self.entity_to_nodes.clear();
        self.node_kinds.clear();
        self.links.clear();
    }

    pub fn len(&self) -> usize {
        self.node_to_entity.len()
    }

    pub fn is_empty(&self) -> bool {
        self.node_to_entity.is_empty()
    }

    // Helper method for removing all links for a node
    fn remove_all_links_for_node(&mut self, world: &mut World, node_id: u64) {
        // Remove as parent
        if let Some(owned_children) = self.links.owned_links.remove(&node_id) {
            for child_id in owned_children {
                self.links.owned_reverse.remove(&child_id);
                if let Some(child_entity) = self.get_entity(child_id) {
                    remove_owned_relationship(world, child_entity);
                }
            }
        }

        if let Some(ref_children) = self.links.ref_links.remove(&node_id) {
            for child_id in ref_children {
                if let Some(parents) = self.links.ref_reverse.get_mut(&child_id) {
                    parents.remove(&node_id);
                }
                if let Some(child_entity) = self.get_entity(child_id) {
                    remove_reference_relationship(world, child_entity);
                }
            }
        }

        // Remove as child
        if let Some(owned_parent) = self.links.owned_reverse.remove(&node_id) {
            if let Some(children) = self.links.owned_links.get_mut(&owned_parent) {
                children.remove(&node_id);
            }
        }

        if let Some(ref_parents) = self.links.ref_reverse.remove(&node_id) {
            for parent_id in ref_parents {
                if let Some(children) = self.links.ref_links.get_mut(&parent_id) {
                    children.remove(&node_id);
                }
            }
        }
    }

    // Link management methods integrated with NodeLinks
    pub fn add_owned_link(&mut self, world: &mut World, parent_id: u64, child_id: u64) {
        self.links
            .owned_links
            .entry(parent_id)
            .or_default()
            .insert(child_id);
        self.links.owned_reverse.insert(child_id, parent_id);

        // Update Bevy relationships
        if let (Some(parent_entity), Some(child_entity)) =
            (self.get_entity(parent_id), self.get_entity(child_id))
        {
            add_owned_relationship(world, parent_entity, child_entity);
        }
    }

    pub fn add_ref_link(&mut self, world: &mut World, parent_id: u64, child_id: u64) {
        self.links
            .ref_links
            .entry(parent_id)
            .or_default()
            .insert(child_id);
        self.links
            .ref_reverse
            .entry(child_id)
            .or_default()
            .insert(parent_id);

        // Update Bevy relationships
        if let (Some(parent_entity), Some(child_entity)) =
            (self.get_entity(parent_id), self.get_entity(child_id))
        {
            add_reference_relationship(world, parent_entity, child_entity);
        }
    }

    pub fn remove_owned_link(&mut self, world: &mut World, parent_id: u64, child_id: u64) {
        if let Some(children) = self.links.owned_links.get_mut(&parent_id) {
            children.remove(&child_id);
        }
        self.links.owned_reverse.remove(&child_id);

        // Update Bevy relationships
        if let Some(child_entity) = self.get_entity(child_id) {
            remove_owned_relationship(world, child_entity);
        }
    }

    pub fn remove_ref_link(&mut self, world: &mut World, parent_id: u64, child_id: u64) {
        if let Some(children) = self.links.ref_links.get_mut(&parent_id) {
            children.remove(&child_id);
        }
        if let Some(parents) = self.links.ref_reverse.get_mut(&child_id) {
            parents.remove(&parent_id);
        }

        // Update Bevy relationships
        if let Some(child_entity) = self.get_entity(child_id) {
            remove_reference_relationship(world, child_entity);
        }
    }

    pub fn get_owned_children(&self, parent_id: u64) -> Vec<u64> {
        self.links.get_owned_children(parent_id)
    }

    pub fn get_ref_children(&self, parent_id: u64) -> Vec<u64> {
        self.links.get_ref_children(parent_id)
    }

    pub fn get_owned_parent(&self, child_id: u64) -> Option<u64> {
        self.links.get_owned_parent(child_id)
    }

    pub fn get_ref_parents(&self, child_id: u64) -> Vec<u64> {
        self.links.get_ref_parents(child_id)
    }

    pub fn has_owned_link(&self, parent_id: u64, child_id: u64) -> bool {
        self.links.has_owned_link(parent_id, child_id)
    }

    pub fn has_ref_link(&self, parent_id: u64, child_id: u64) -> bool {
        self.links.has_ref_link(parent_id, child_id)
    }

    // Get children of specific kind by combining relationship data with kind filtering
    pub fn get_children_of_kind(&self, parent_id: u64, kind: NodeKind) -> Vec<u64> {
        let mut children = Vec::new();

        // Get owned children of the specified kind
        for child_id in self.get_owned_children(parent_id) {
            if let Some(child_kind) = self.node_kinds.get(&child_id) {
                if *child_kind == kind {
                    children.push(child_id);
                }
            }
        }

        // Get referenced children of the specified kind
        for child_id in self.get_ref_children(parent_id) {
            if let Some(child_kind) = self.node_kinds.get(&child_id) {
                if *child_kind == kind && !children.contains(&child_id) {
                    children.push(child_id);
                }
            }
        }

        children
    }

    // Get parents of specific kind
    pub fn get_parents_of_kind(&self, child_id: u64, kind: NodeKind) -> Vec<u64> {
        let mut parents = Vec::new();

        // Check owned parent
        if let Some(parent_id) = self.get_owned_parent(child_id) {
            if let Some(parent_kind) = self.node_kinds.get(&parent_id) {
                if *parent_kind == kind {
                    parents.push(parent_id);
                }
            }
        }

        // Check reference parents
        for parent_id in self.get_ref_parents(child_id) {
            if let Some(parent_kind) = self.node_kinds.get(&parent_id) {
                if *parent_kind == kind && !parents.contains(&parent_id) {
                    parents.push(parent_id);
                }
            }
        }

        parents
    }
}

// Enhanced node links tracking with relationship integration
#[derive(Resource, Default, Clone)]
pub struct NodeLinks {
    // Owned relationships (parent owns child)
    pub owned_links: HashMap<u64, HashSet<u64>>,
    pub owned_reverse: HashMap<u64, u64>,

    // Reference relationships (parent references child)
    pub ref_links: HashMap<u64, HashSet<u64>>,
    pub ref_reverse: HashMap<u64, HashSet<u64>>,
}

impl NodeLinks {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_owned_link(
        &mut self,
        world: &mut World,
        parent_id: u64,
        child_id: u64,
        node_map: &SmartNodeMap,
    ) {
        // Update our tracking
        self.owned_links
            .entry(parent_id)
            .or_default()
            .insert(child_id);
        self.owned_reverse.insert(child_id, parent_id);

        // Update Bevy relationships
        if let (Some(parent_entity), Some(child_entity)) = (
            node_map.get_entity(parent_id),
            node_map.get_entity(child_id),
        ) {
            add_owned_relationship(world, parent_entity, child_entity);
        }
    }

    pub fn add_ref_link(
        &mut self,
        world: &mut World,
        parent_id: u64,
        child_id: u64,
        node_map: &SmartNodeMap,
    ) {
        // Update our tracking
        self.ref_links
            .entry(parent_id)
            .or_default()
            .insert(child_id);
        self.ref_reverse
            .entry(child_id)
            .or_default()
            .insert(parent_id);

        // Update Bevy relationships
        if let (Some(parent_entity), Some(child_entity)) = (
            node_map.get_entity(parent_id),
            node_map.get_entity(child_id),
        ) {
            add_reference_relationship(world, parent_entity, child_entity);
        }
    }

    pub fn remove_owned_link(
        &mut self,
        world: &mut World,
        parent_id: u64,
        child_id: u64,
        node_map: &SmartNodeMap,
    ) {
        if let Some(children) = self.owned_links.get_mut(&parent_id) {
            children.remove(&child_id);
        }
        self.owned_reverse.remove(&child_id);

        // Update Bevy relationships
        if let Some(child_entity) = node_map.get_entity(child_id) {
            remove_owned_relationship(world, child_entity);
        }
    }

    pub fn remove_ref_link(
        &mut self,
        world: &mut World,
        parent_id: u64,
        child_id: u64,
        node_map: &SmartNodeMap,
    ) {
        if let Some(children) = self.ref_links.get_mut(&parent_id) {
            children.remove(&child_id);
        }
        if let Some(parents) = self.ref_reverse.get_mut(&child_id) {
            parents.remove(&parent_id);
        }

        // Update Bevy relationships
        if let Some(child_entity) = node_map.get_entity(child_id) {
            remove_reference_relationship(world, child_entity);
        }
    }

    pub fn get_owned_children(&self, parent_id: u64) -> Vec<u64> {
        self.owned_links
            .get(&parent_id)
            .map(|set| set.iter().copied().collect())
            .unwrap_or_default()
    }

    pub fn get_ref_children(&self, parent_id: u64) -> Vec<u64> {
        self.ref_links
            .get(&parent_id)
            .map(|set| set.iter().copied().collect())
            .unwrap_or_default()
    }

    pub fn get_owned_parent(&self, child_id: u64) -> Option<u64> {
        self.owned_reverse.get(&child_id).copied()
    }

    pub fn get_ref_parents(&self, child_id: u64) -> Vec<u64> {
        self.ref_reverse
            .get(&child_id)
            .map(|set| set.iter().copied().collect())
            .unwrap_or_default()
    }

    pub fn has_owned_link(&self, parent_id: u64, child_id: u64) -> bool {
        self.owned_links
            .get(&parent_id)
            .map(|children| children.contains(&child_id))
            .unwrap_or(false)
    }

    pub fn has_ref_link(&self, parent_id: u64, child_id: u64) -> bool {
        self.ref_links
            .get(&parent_id)
            .map(|children| children.contains(&child_id))
            .unwrap_or(false)
    }

    pub fn remove_all_links(&mut self, world: &mut World, node_id: u64, node_map: &SmartNodeMap) {
        // Remove as parent
        if let Some(owned_children) = self.owned_links.remove(&node_id) {
            for child_id in owned_children {
                self.owned_reverse.remove(&child_id);
                if let Some(child_entity) = node_map.get_entity(child_id) {
                    remove_owned_relationship(world, child_entity);
                }
            }
        }

        if let Some(ref_children) = self.ref_links.remove(&node_id) {
            for child_id in ref_children {
                if let Some(parents) = self.ref_reverse.get_mut(&child_id) {
                    parents.remove(&node_id);
                }
                if let Some(child_entity) = node_map.get_entity(child_id) {
                    remove_reference_relationship(world, child_entity);
                }
            }
        }

        // Remove as child
        if let Some(owned_parent) = self.owned_reverse.remove(&node_id) {
            if let Some(children) = self.owned_links.get_mut(&owned_parent) {
                children.remove(&node_id);
            }
        }

        if let Some(ref_parents) = self.ref_reverse.remove(&node_id) {
            for parent_id in ref_parents {
                if let Some(children) = self.ref_links.get_mut(&parent_id) {
                    children.remove(&node_id);
                }
            }
        }
    }

    pub fn clear(&mut self) {
        self.owned_links.clear();
        self.owned_reverse.clear();
        self.ref_links.clear();
        self.ref_reverse.clear();
    }
}
