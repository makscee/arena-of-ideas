use crate::prelude::*;
use bevy::prelude::*;
use schema::{Context, ContextSource, NodeError, NodeResult};
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

/// Unified WorldSource enum for both immutable and mutable World access
pub enum WorldSource<'w> {
    Immutable(&'w World),
    Mutable(&'w mut World),
    None,
}

impl<'w> WorldSource<'w> {
    pub fn new_immutable(world: &'w World) -> Self {
        Self::Immutable(world)
    }

    pub fn new_mutable(world: &'w mut World) -> Self {
        Self::Mutable(world)
    }

    pub const fn new_empty() -> Self {
        Self::None
    }

    fn world(&self) -> &World {
        match self {
            Self::Immutable(world) => world,
            Self::Mutable(world) => world,
            Self::None => panic!(),
        }
    }

    fn world_mut(&mut self) -> Option<&mut World> {
        match self {
            Self::Immutable(_) | Self::None => None,
            Self::Mutable(world) => Some(world),
        }
    }
}

impl<'w> ContextSource for WorldSource<'w> {
    fn get_node_kind(&self, id: u64) -> NodeResult<NodeKind> {
        let world = self.world();
        if let Some(map) = world.get_resource::<NodeEntityMap>() {
            if let Some(entity) = map.get_entity(id) {
                if let Some(node) = world.get::<NodeEntity>(entity) {
                    return Ok(node.kind);
                }
            }
        }
        Err(NodeError::NotFound(id))
    }

    fn get_children(&self, from_id: u64) -> NodeResult<Vec<u64>> {
        let world = self.world();
        if let Some(links) = world.get_resource::<NodeLinks>() {
            Ok(links.get_children(from_id))
        } else {
            Ok(Vec::new())
        }
    }

    fn get_children_of_kind(&self, from_id: u64, kind: NodeKind) -> NodeResult<Vec<u64>> {
        let world = self.world();
        if let Some(links) = world.get_resource::<NodeLinks>() {
            Ok(links.get_children_of_kind(from_id, kind))
        } else {
            Ok(Vec::new())
        }
    }

    fn get_parents(&self, id: u64) -> NodeResult<Vec<u64>> {
        let world = self.world();
        if let Some(links) = world.get_resource::<NodeLinks>() {
            Ok(links.get_parents(id))
        } else {
            Ok(Vec::new())
        }
    }

    fn get_parents_of_kind(&self, id: u64, kind: NodeKind) -> NodeResult<Vec<u64>> {
        let world = self.world();
        if let Some(links) = world.get_resource::<NodeLinks>() {
            Ok(links.get_parents_of_kind(id, kind))
        } else {
            Ok(Vec::new())
        }
    }

    fn add_link(&mut self, from_id: u64, to_id: u64) -> NodeResult<()> {
        let to_kind = self.get_node_kind(to_id)?;
        if let Some(world) = self.world_mut() {
            if let Some(mut links) = world.get_resource_mut::<NodeLinks>() {
                links.add_link(from_id, to_id, to_kind);
                Ok(())
            } else {
                Err(NodeError::ContextError(anyhow::anyhow!(
                    "NodeLinks resource not found"
                )))
            }
        } else {
            Err(NodeError::ContextError(anyhow::anyhow!(
                "Cannot modify links with immutable WorldSource"
            )))
        }
    }

    fn remove_link(&mut self, from_id: u64, to_id: u64) -> NodeResult<()> {
        if let Some(world) = self.world_mut() {
            if let Some(mut links) = world.get_resource_mut::<NodeLinks>() {
                links.remove_link(from_id, to_id);
                Ok(())
            } else {
                Err(NodeError::ContextError(anyhow::anyhow!(
                    "NodeLinks resource not found"
                )))
            }
        } else {
            Err(NodeError::ContextError(anyhow::anyhow!(
                "Cannot modify links with immutable WorldSource"
            )))
        }
    }

    fn is_linked(&self, from_id: u64, to_id: u64) -> NodeResult<bool> {
        let world = self.world();
        if let Some(links) = world.get_resource::<NodeLinks>() {
            Ok(links.has_link(from_id, to_id))
        } else {
            Ok(false)
        }
    }
}

/// Extension trait for Context to load nodes in client
pub trait ClientContextExt {
    fn load<'a, T>(&'a self, id: u64) -> NodeResult<&'a T>
    where
        T: 'static + ClientNode;

    fn load_many<'a, T>(&'a self, ids: &Vec<u64>) -> NodeResult<Vec<&'a T>>
    where
        T: 'static + ClientNode;

    fn load_children<'a, T: ClientNode>(&'a self, from_id: u64) -> NodeResult<Vec<&'a T>>;
    fn world<'a>(&'a self) -> Option<&'a World>;
    fn world_mut<'a>(&'a mut self) -> Option<&'a mut World>;
}

impl<'w> ClientContextExt for Context<WorldSource<'w>> {
    fn load<'a, T>(&'a self, id: u64) -> NodeResult<&'a T>
    where
        T: 'static + ClientNode,
    {
        let world = self.source().world();
        if let Some(map) = world.get_resource::<NodeEntityMap>() {
            if let Some(entity) = map.get_entity(id) {
                if let Some(component) = world.get::<T>(entity) {
                    return Ok(component);
                } else {
                    return Err(NodeError::LoadError(
                        "Failed to get component from entity".into(),
                    ));
                }
            }
        }
        Err(NodeError::NotFound(id))
    }

    fn load_many<'a, T>(&'a self, ids: &Vec<u64>) -> NodeResult<Vec<&'a T>>
    where
        T: 'static + ClientNode,
    {
        let mut results = Vec::new();
        for id in ids {
            results.push(self.load::<T>(*id)?);
        }
        Ok(results)
    }

    fn load_children<'a, T: ClientNode>(&'a self, from_id: u64) -> NodeResult<Vec<&'a T>> {
        let kind = T::kind_s();
        let ids = self.get_children_of_kind(from_id, kind)?;
        self.load_many(&ids)
    }

    fn world<'a>(&'a self) -> Option<&'a World> {
        Some(self.source().world())
    }

    fn world_mut<'a>(&'a mut self) -> Option<&'a mut World> {
        self.source_mut().world_mut()
    }
}

/// Extension for using Context with Bevy World
pub trait WorldContextExt {
    /// Execute with a context using this world as the source (immutable)
    fn with_context<R, F>(&self, f: F) -> R
    where
        F: FnOnce(&mut Context<WorldSource<'_>>) -> R;

    /// Execute with a context using this world as the source (mutable)
    fn with_context_mut<R, F>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Context<WorldSource<'_>>) -> R;
}

impl WorldContextExt for World {
    fn with_context<R, F>(&self, f: F) -> R
    where
        F: FnOnce(&mut Context<WorldSource<'_>>) -> R,
    {
        let source = WorldSource::new_immutable(self);
        Context::exec(source, f)
    }

    fn with_context_mut<R, F>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Context<WorldSource<'_>>) -> R,
    {
        let source = WorldSource::new_mutable(self);
        Context::exec(source, f)
    }
}

/// Type alias for convenience
pub type ClientContext<'w> = Context<WorldSource<'w>>;

pub const EMPTY_CONTEXT: ClientContext = Context::new(WorldSource::new_empty());
