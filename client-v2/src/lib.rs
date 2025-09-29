pub use schema_v2::*;

// Re-export Component, Owned, Ref from schema
pub use schema_v2::{Component, Context, ContextLayer, ContextSource, Owned, Ref};

// Re-export generated code
include!(concat!(env!("OUT_DIR"), "/client_nodes.rs"));

use bevy::prelude::Component as BevyComponent;
use bevy::prelude::*;
use std::collections::HashMap;

/// Resource for mapping node IDs to entities
#[derive(Resource, Default)]
pub struct NodeEntityMap {
    id_to_entity: HashMap<u64, Entity>,
    entity_to_id: HashMap<Entity, u64>,
}

impl NodeEntityMap {
    pub fn insert(&mut self, id: u64, entity: Entity) {
        self.id_to_entity.insert(id, entity);
        self.entity_to_id.insert(entity, id);
    }

    pub fn get_entity(&self, id: u64) -> Option<Entity> {
        self.id_to_entity.get(&id).copied()
    }

    pub fn get_id(&self, entity: Entity) -> Option<u64> {
        self.entity_to_id.get(&entity).copied()
    }

    pub fn remove_by_id(&mut self, id: u64) -> Option<Entity> {
        if let Some(entity) = self.id_to_entity.remove(&id) {
            self.entity_to_id.remove(&entity);
            Some(entity)
        } else {
            None
        }
    }

    pub fn remove_by_entity(&mut self, entity: Entity) -> Option<u64> {
        if let Some(id) = self.entity_to_id.remove(&entity) {
            self.id_to_entity.remove(&id);
            Some(id)
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.id_to_entity.clear();
        self.entity_to_id.clear();
    }
}

/// Resource for tracking node links in the client
#[derive(Resource, Default)]
pub struct NodeLinks {
    links: HashMap<u64, Vec<(u64, NodeKind)>>, // (to_id, to_kind)
    reverse_links: HashMap<u64, Vec<(u64, NodeKind)>>, // child -> parents
}

impl NodeLinks {
    pub fn add_link(&mut self, from_id: u64, to_id: u64, to_kind: NodeKind) {
        self.links
            .entry(from_id)
            .or_insert_with(Vec::new)
            .push((to_id, to_kind));

        self.reverse_links
            .entry(to_id)
            .or_insert_with(Vec::new)
            .push((from_id, to_kind));
    }

    pub fn get_children(&self, from_id: u64) -> Vec<u64> {
        self.links
            .get(&from_id)
            .map(|links| links.iter().map(|(id, _)| *id).collect())
            .unwrap_or_default()
    }

    pub fn get_children_of_kind(&self, from_id: u64, kind: NodeKind) -> Vec<u64> {
        self.links
            .get(&from_id)
            .map(|links| {
                links
                    .iter()
                    .filter(|(_, node_kind)| *node_kind == kind)
                    .map(|(id, _)| *id)
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_parents(&self, child_id: u64) -> Vec<u64> {
        self.reverse_links
            .get(&child_id)
            .map(|parents| parents.iter().map(|(id, _)| *id).collect())
            .unwrap_or_default()
    }

    pub fn get_parents_of_kind(&self, child_id: u64, kind: NodeKind) -> Vec<u64> {
        self.reverse_links
            .get(&child_id)
            .map(|parents| {
                parents
                    .iter()
                    .filter(|(_, node_kind)| *node_kind == kind)
                    .map(|(id, _)| *id)
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn has_link(&self, from_id: u64, to_id: u64) -> bool {
        self.links
            .get(&from_id)
            .map(|links| links.iter().any(|(id, _)| *id == to_id))
            .unwrap_or(false)
    }

    pub fn remove_link(&mut self, from_id: u64, to_id: u64) {
        if let Some(links) = self.links.get_mut(&from_id) {
            links.retain(|(id, _)| *id != to_id);
        }

        if let Some(parents) = self.reverse_links.get_mut(&to_id) {
            parents.retain(|(id, _)| *id != from_id);
        }
    }

    pub fn clear(&mut self) {
        self.links.clear();
        self.reverse_links.clear();
    }
}

/// Marker component for entities with nodes
#[derive(Component)]
pub struct NodeEntity {
    pub id: u64,
    pub kind: NodeKind,
}

/// ContextSource implementation for immutable World access
pub struct WorldSource<'w> {
    world: &'w World,
}

impl<'w> WorldSource<'w> {
    pub fn new(world: &'w World) -> Self {
        Self { world }
    }
}

impl<'w> ContextSource for WorldSource<'w> {
    fn get_node_kind(&self, id: u64) -> NodeResult<NodeKind> {
        if let Some(map) = self.world.get_resource::<NodeEntityMap>() {
            if let Some(entity) = map.get_entity(id) {
                if let Some(node) = self.world.get::<NodeEntity>(entity) {
                    return Ok(node.kind);
                }
            }
        }
        Err(NodeError::NotFound(id))
    }

    fn get_children(&self, from_id: u64) -> NodeResult<Vec<u64>> {
        if let Some(links) = self.world.get_resource::<NodeLinks>() {
            Ok(links.get_children(from_id))
        } else {
            Ok(Vec::new())
        }
    }

    fn get_children_of_kind(&self, from_id: u64, kind: NodeKind) -> NodeResult<Vec<u64>> {
        if let Some(links) = self.world.get_resource::<NodeLinks>() {
            Ok(links.get_children_of_kind(from_id, kind))
        } else {
            Ok(Vec::new())
        }
    }

    fn get_parents(&self, id: u64) -> NodeResult<Vec<u64>> {
        if let Some(links) = self.world.get_resource::<NodeLinks>() {
            Ok(links.get_parents(id))
        } else {
            Ok(Vec::new())
        }
    }

    fn get_parents_of_kind(&self, id: u64, kind: NodeKind) -> NodeResult<Vec<u64>> {
        if let Some(links) = self.world.get_resource::<NodeLinks>() {
            Ok(links.get_parents_of_kind(id, kind))
        } else {
            Ok(Vec::new())
        }
    }

    fn add_link(&mut self, _from_id: u64, _to_id: u64) -> NodeResult<()> {
        Err(NodeError::ContextError(anyhow::anyhow!(
            "Cannot modify links with immutable WorldSource"
        )))
    }

    fn remove_link(&mut self, _from_id: u64, _to_id: u64) -> NodeResult<()> {
        Err(NodeError::ContextError(anyhow::anyhow!(
            "Cannot modify links with immutable WorldSource"
        )))
    }

    fn is_linked(&self, from_id: u64, to_id: u64) -> NodeResult<bool> {
        if let Some(links) = self.world.get_resource::<NodeLinks>() {
            Ok(links.has_link(from_id, to_id))
        } else {
            Ok(false)
        }
    }
}

/// ContextSource implementation for mutable World access
pub struct MutWorldSource<'w> {
    world: &'w mut World,
}

impl<'w> MutWorldSource<'w> {
    pub fn new(world: &'w mut World) -> Self {
        Self { world }
    }

    pub fn world(&self) -> &World {
        self.world
    }

    pub fn world_mut(&mut self) -> &mut World {
        self.world
    }
}

impl<'w> ContextSource for MutWorldSource<'w> {
    fn get_node_kind(&self, id: u64) -> NodeResult<NodeKind> {
        if let Some(map) = self.world.get_resource::<NodeEntityMap>() {
            if let Some(entity) = map.get_entity(id) {
                if let Some(node) = self.world.get::<NodeEntity>(entity) {
                    return Ok(node.kind);
                }
            }
        }
        Err(NodeError::NotFound(id))
    }

    fn get_children(&self, from_id: u64) -> NodeResult<Vec<u64>> {
        if let Some(links) = self.world.get_resource::<NodeLinks>() {
            Ok(links.get_children(from_id))
        } else {
            Ok(Vec::new())
        }
    }

    fn get_children_of_kind(&self, from_id: u64, kind: NodeKind) -> NodeResult<Vec<u64>> {
        if let Some(links) = self.world.get_resource::<NodeLinks>() {
            Ok(links.get_children_of_kind(from_id, kind))
        } else {
            Ok(Vec::new())
        }
    }

    fn get_parents(&self, id: u64) -> NodeResult<Vec<u64>> {
        if let Some(links) = self.world.get_resource::<NodeLinks>() {
            Ok(links.get_parents(id))
        } else {
            Ok(Vec::new())
        }
    }

    fn get_parents_of_kind(&self, id: u64, kind: NodeKind) -> NodeResult<Vec<u64>> {
        if let Some(links) = self.world.get_resource::<NodeLinks>() {
            Ok(links.get_parents_of_kind(id, kind))
        } else {
            Ok(Vec::new())
        }
    }

    fn add_link(&mut self, from_id: u64, to_id: u64) -> NodeResult<()> {
        // Get the to_kind from the target node
        let to_kind = self.get_node_kind(to_id)?;

        if let Some(mut links) = self.world.get_resource_mut::<NodeLinks>() {
            links.add_link(from_id, to_id, to_kind);
            Ok(())
        } else {
            Err(NodeError::ContextError(anyhow::anyhow!(
                "NodeLinks resource not found"
            )))
        }
    }

    fn remove_link(&mut self, from_id: u64, to_id: u64) -> NodeResult<()> {
        if let Some(mut links) = self.world.get_resource_mut::<NodeLinks>() {
            links.remove_link(from_id, to_id);
            Ok(())
        } else {
            Err(NodeError::ContextError(anyhow::anyhow!(
                "NodeLinks resource not found"
            )))
        }
    }

    fn is_linked(&self, from_id: u64, to_id: u64) -> NodeResult<bool> {
        if let Some(links) = self.world.get_resource::<NodeLinks>() {
            Ok(links.has_link(from_id, to_id))
        } else {
            Ok(false)
        }
    }
}

/// Extension trait for Context to load nodes in client
pub trait ClientContextExt<S: ContextSource> {
    /// Load a node by ID - requires access to World through the source
    fn load<T>(&mut self, id: u64) -> NodeResult<T>
    where
        T: 'static + HasNodeKind + BevyComponent + Clone;

    /// Load multiple nodes
    fn load_many<T>(&mut self, ids: &[u64]) -> NodeResult<Vec<T>>
    where
        T: 'static + HasNodeKind + BevyComponent + Clone;

    /// Load linked nodes
    fn load_linked<T>(&mut self, from_id: u64) -> NodeResult<Vec<T>>
    where
        T: 'static + HasNodeKind + BevyComponent + Clone;
}

impl<'w> ClientContextExt<WorldSource<'w>> for Context<WorldSource<'w>> {
    fn load<T>(&mut self, id: u64) -> NodeResult<T>
    where
        T: 'static + HasNodeKind + BevyComponent + Clone,
    {
        let world = self.source().world;
        if let Some(map) = world.get_resource::<NodeEntityMap>() {
            if let Some(entity) = map.get_entity(id) {
                if let Some(component) = world.get::<T>(entity) {
                    return Ok(component.clone());
                } else {
                    return Err(NodeError::CastError);
                }
            }
        }
        Err(NodeError::NotFound(id))
    }

    fn load_many<T>(&mut self, ids: &[u64]) -> NodeResult<Vec<T>>
    where
        T: 'static + HasNodeKind + BevyComponent + Clone,
    {
        let mut results = Vec::new();
        for id in ids {
            results.push(self.load::<T>(*id)?);
        }
        Ok(results)
    }

    fn load_linked<T>(&mut self, from_id: u64) -> NodeResult<Vec<T>>
    where
        T: 'static + HasNodeKind + BevyComponent + Clone,
    {
        let kind = T::node_kind();
        let ids = self.get_children_of_kind(from_id, kind)?;
        self.load_many(&ids)
    }
}

impl<'w> ClientContextExt<MutWorldSource<'w>> for Context<MutWorldSource<'w>> {
    fn load<T>(&mut self, id: u64) -> NodeResult<T>
    where
        T: 'static + HasNodeKind + BevyComponent + Clone,
    {
        let world = self.source().world();
        if let Some(map) = world.get_resource::<NodeEntityMap>() {
            if let Some(entity) = map.get_entity(id) {
                if let Some(component) = world.get::<T>(entity) {
                    return Ok(component.clone());
                } else {
                    return Err(NodeError::CastError);
                }
            }
        }
        Err(NodeError::NotFound(id))
    }

    fn load_many<T>(&mut self, ids: &[u64]) -> NodeResult<Vec<T>>
    where
        T: 'static + HasNodeKind + BevyComponent + Clone,
    {
        let mut results = Vec::new();
        for id in ids {
            results.push(self.load::<T>(*id)?);
        }
        Ok(results)
    }

    fn load_linked<T>(&mut self, from_id: u64) -> NodeResult<Vec<T>>
    where
        T: 'static + HasNodeKind + BevyComponent + Clone,
    {
        let kind = T::node_kind();
        let ids = self.get_children_of_kind(from_id, kind)?;
        self.load_many(&ids)
    }
}

/// Extension for using Context with Bevy World
pub trait WorldContextExt {
    /// Execute with a context using this world as the source (immutable)
    fn with_context<R, F>(&self, f: F) -> R
    where
        F: FnOnce(&mut Context<WorldSource>) -> R;

    /// Execute with a context using this world as the source (mutable)
    fn with_context_mut<R, F>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Context<MutWorldSource>) -> R;
}

impl WorldContextExt for World {
    fn with_context<R, F>(&self, f: F) -> R
    where
        F: FnOnce(&mut Context<WorldSource>) -> R,
    {
        let source = WorldSource::new(self);
        Context::exec(source, f)
    }

    fn with_context_mut<R, F>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Context<MutWorldSource>) -> R,
    {
        let source = MutWorldSource::new(self);
        Context::exec(source, f)
    }
}

/// Plugin to add node system resources
pub struct NodeSystemPlugin;

impl Plugin for NodeSystemPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(NodeEntityMap::default())
            .insert_resource(NodeLinks::default());
    }
}
