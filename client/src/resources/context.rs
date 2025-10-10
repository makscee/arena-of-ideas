use super::*;
use bevy::ecs::component::Mutable;
use schema::{Context, ContextSource, NodeError, NodeResult};
use std::collections::HashMap;

/// Resource for mapping node IDs to entities
#[derive(Resource, Default)]
pub struct NodeEntityMap {
    id_to_entity: HashMap<u64, Entity>,
    entity_to_ids: HashMap<Entity, Vec<u64>>,
}

impl NodeEntityMap {
    pub fn insert(&mut self, id: u64, entity: Entity) {
        self.id_to_entity.insert(id, entity);
        self.entity_to_ids
            .entry(entity)
            .or_insert_with(Vec::new)
            .push(id);
    }

    pub fn get_entity(&self, id: u64) -> Option<Entity> {
        self.id_to_entity.get(&id).copied()
    }

    pub fn get_ids(&self, entity: Entity) -> Vec<u64> {
        self.entity_to_ids.get(&entity).cloned().unwrap_or_default()
    }

    pub fn get_id(&self, entity: Entity) -> Option<u64> {
        self.entity_to_ids
            .get(&entity)
            .and_then(|ids| ids.first().copied())
    }

    pub fn remove_by_id(&mut self, id: u64) -> Option<Entity> {
        if let Some(entity) = self.id_to_entity.remove(&id) {
            if let Some(ids) = self.entity_to_ids.get_mut(&entity) {
                ids.retain(|&existing_id| existing_id != id);
                if ids.is_empty() {
                    self.entity_to_ids.remove(&entity);
                }
            }
            Some(entity)
        } else {
            None
        }
    }

    pub fn remove_by_entity(&mut self, entity: Entity) -> Vec<u64> {
        if let Some(ids) = self.entity_to_ids.remove(&entity) {
            for id in &ids {
                self.id_to_entity.remove(id);
            }
            ids
        } else {
            Vec::new()
        }
    }

    pub fn clear(&mut self) {
        self.id_to_entity.clear();
        self.entity_to_ids.clear();
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
#[derive(BevyComponent)]
pub struct NodeEntity {
    pub nodes: Vec<(u64, NodeKind)>,
}

impl NodeEntity {
    pub fn new(id: u64, kind: NodeKind) -> Self {
        Self {
            nodes: vec![(id, kind)],
        }
    }

    pub fn with_nodes(nodes: Vec<(u64, NodeKind)>) -> Self {
        Self { nodes }
    }

    pub fn add_node(&mut self, id: u64, kind: NodeKind) {
        self.nodes.push((id, kind));
    }

    pub fn get_node_kinds(&self) -> Vec<NodeKind> {
        self.nodes.iter().map(|(_, kind)| *kind).collect()
    }

    pub fn get_node_ids(&self) -> Vec<u64> {
        self.nodes.iter().map(|(id, _)| *id).collect()
    }

    pub fn has_kind(&self, kind: NodeKind) -> bool {
        self.nodes.iter().any(|(_, k)| *k == kind)
    }
}

/// Unified WorldSource enum for both immutable and mutable World access
pub enum WorldSource<'w> {
    Immutable(&'w World),
    Mutable(&'w mut World),
    Battle(&'w mut BattleSimulation),
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

    pub fn new_battle(battle: &'w mut BattleSimulation) -> Self {
        Self::Battle(battle)
    }

    pub fn battle(&self) -> NodeResult<&BattleSimulation> {
        match self {
            Self::Battle(battle) => Ok(battle),
            _ => Err(NodeError::custom("Source is not a BattleSimulation")),
        }
    }

    pub fn battle_mut(&mut self) -> NodeResult<&mut BattleSimulation> {
        match self {
            Self::Battle(battle) => Ok(battle),
            _ => Err(NodeError::custom("Source is not a BattleSimulation")),
        }
    }

    fn get_rng(&mut self) -> Option<&mut ChaCha8Rng> {
        match self {
            Self::Battle(battle) => Some(&mut battle.rng),
            _ => None,
        }
    }

    pub fn world(&self) -> NodeResult<&World> {
        match self {
            Self::Immutable(world) => Ok(world),
            Self::Mutable(world) => Ok(world),
            Self::Battle(battle) => Ok(&battle.world),
            Self::None => Err(NodeError::custom("Source World not set")),
        }
    }

    pub fn world_mut(&mut self) -> NodeResult<&mut World> {
        match self {
            Self::Immutable(_) => Err(NodeError::custom("Source World is immutable")),
            Self::None => Err(NodeError::custom("Source World not set")),
            Self::Mutable(world) => Ok(world),
            Self::Battle(battle) => Ok(&mut battle.world),
        }
    }
}

impl<'w> ContextSource for WorldSource<'w> {
    fn get_node_kind(&self, id: u64) -> NodeResult<NodeKind> {
        let world = self.world()?;
        if let Some(map) = world.get_resource::<NodeEntityMap>() {
            if let Some(entity) = map.get_entity(id) {
                if let Some(node) = world.get::<NodeEntity>(entity) {
                    for (node_id, kind) in &node.nodes {
                        if *node_id == id {
                            return Ok(*kind);
                        }
                    }
                }
            }
        }
        Err(NodeError::not_found(id))
    }

    fn get_children(&self, from_id: u64) -> NodeResult<Vec<u64>> {
        let world = self.world()?;
        if let Some(links) = world.get_resource::<NodeLinks>() {
            Ok(links.get_children(from_id))
        } else {
            Ok(Vec::new())
        }
    }

    fn get_children_of_kind(&self, from_id: u64, kind: NodeKind) -> NodeResult<Vec<u64>> {
        let world = self.world()?;
        if let Some(links) = world.get_resource::<NodeLinks>() {
            Ok(links.get_children_of_kind(from_id, kind))
        } else {
            Ok(Vec::new())
        }
    }

    fn get_parents(&self, id: u64) -> NodeResult<Vec<u64>> {
        let world = self.world()?;
        if let Some(links) = world.get_resource::<NodeLinks>() {
            Ok(links.get_parents(id))
        } else {
            Ok(Vec::new())
        }
    }

    fn get_parents_of_kind(&self, id: u64, kind: NodeKind) -> NodeResult<Vec<u64>> {
        let world = self.world()?;
        if let Some(links) = world.get_resource::<NodeLinks>() {
            Ok(links.get_parents_of_kind(id, kind))
        } else {
            Ok(Vec::new())
        }
    }

    fn add_link(&mut self, from_id: u64, to_id: u64) -> NodeResult<()> {
        let to_kind = self.get_node_kind(to_id)?;
        if let Ok(world) = self.world_mut() {
            if let Some(mut links) = world.get_resource_mut::<NodeLinks>() {
                links.add_link(from_id, to_id, to_kind);
                Ok(())
            } else {
                Err(NodeError::context_error(anyhow::anyhow!(
                    "NodeLinks resource not found"
                )))
            }
        } else {
            Err(NodeError::context_error(anyhow::anyhow!(
                "Cannot modify links with immutable WorldSource"
            )))
        }
    }

    fn remove_link(&mut self, from_id: u64, to_id: u64) -> NodeResult<()> {
        if let Ok(world) = self.world_mut() {
            if let Some(mut links) = world.get_resource_mut::<NodeLinks>() {
                links.remove_link(from_id, to_id);
                Ok(())
            } else {
                Err(NodeError::context_error(anyhow::anyhow!(
                    "NodeLinks resource not found"
                )))
            }
        } else {
            Err(NodeError::context_error(anyhow::anyhow!(
                "Cannot modify links with immutable WorldSource"
            )))
        }
    }

    fn is_linked(&self, from_id: u64, to_id: u64) -> NodeResult<bool> {
        let world = self.world()?;
        if let Some(links) = world.get_resource::<NodeLinks>() {
            Ok(links.has_link(from_id, to_id))
        } else {
            Ok(false)
        }
    }

    fn set_var(&mut self, node_id: u64, var: VarName, value: VarValue) -> NodeResult<()> {
        self.load_and_set_var(self.get_node_kind(node_id)?, node_id, var, value.clone())?;
        let world = self.world_mut()?;
        if let Some(map) = world.get_resource::<NodeEntityMap>() {
            if let Some(entity) = map.get_entity(node_id) {
                if let Some(mut node_state) = world.get_mut::<NodeState>(entity) {
                    node_state.init(var, value);
                    Ok(())
                } else {
                    Err(NodeError::custom(format!(
                        "NodeState not found for {node_id}"
                    )))
                }
            } else {
                Err(NodeError::custom(format!("Entity not found for {node_id}")))
            }
        } else {
            Err(NodeError::custom("Failed to get NodeEntityMap"))
        }
    }

    fn get_var_direct(&self, node_id: u64, var: VarName) -> NodeResult<VarValue> {
        self.load_and_get_var(self.get_node_kind(node_id)?, node_id, var)
    }
}

/// Extension trait for Context to load nodes in client
pub trait ClientContextExt {
    fn rng(&mut self) -> NodeResult<&mut ChaCha8Rng>;
    fn color(&self, ui: &mut Ui) -> Color32;
    fn load<'a, T: BevyComponent>(&'a self, id: u64) -> NodeResult<&'a T>;
    fn load_entity<'a, T: BevyComponent>(&'a self, entity: Entity) -> NodeResult<&'a T>;
    fn load_mut<'a, T: BevyComponent<Mutability = Mutable>>(
        &'a mut self,
        id: u64,
    ) -> NodeResult<Mut<'a, T>>;
    fn load_entity_mut<'a, T: BevyComponent<Mutability = Mutable>>(
        &'a mut self,
        entity: Entity,
    ) -> NodeResult<Mut<'a, T>>;
    fn load_many<'a, T: BevyComponent>(&'a self, ids: &Vec<u64>) -> NodeResult<Vec<&'a T>>;
    fn load_children<'a, T: ClientNode>(&'a self, from_id: u64) -> NodeResult<Vec<&'a T>>;
    fn world<'a>(&'a self) -> NodeResult<&'a World>;
    fn world_mut<'a>(&'a mut self) -> NodeResult<&'a mut World>;
    fn battle<'a>(&'a self) -> NodeResult<&'a BattleSimulation>;
    fn battle_mut<'a>(&'a mut self) -> NodeResult<&'a mut BattleSimulation>;
    fn t(&self) -> NodeResult<f32>;
    fn t_mut(&mut self) -> NodeResult<&mut f32>;
    fn id(&self, entity: Entity) -> NodeResult<u64>;
    fn entity(&self, id: u64) -> NodeResult<Entity>;
    fn add_id_entity_link(&mut self, id: u64, entity: Entity) -> NodeResult<()>;
    fn remove_id_entity_link(&mut self, id: u64) -> NodeResult<Entity>;
    fn add_link_entities(&mut self, parent: Entity, child: Entity) -> NodeResult<()>;
    fn despawn(&mut self, id: u64) -> NodeResult<()>;
    fn collect_children<'a, T: ClientNode>(&'a self, id: u64) -> NodeResult<Vec<&'a T>>;
    fn owner_entity(&self) -> NodeResult<Entity>;

    // Load versions of new helper functions
    fn load_first_parent<'a, T: ClientNode>(&'a self, id: u64) -> NodeResult<&'a T>;
    fn load_first_parent_recursive<'a, T: ClientNode>(&'a self, id: u64) -> NodeResult<&'a T>;
    fn load_first_child<'a, T: ClientNode>(&'a self, id: u64) -> NodeResult<&'a T>;
    fn load_first_child_recursive<'a, T: ClientNode>(&'a self, id: u64) -> NodeResult<&'a T>;
    fn load_collect_children<'a, T: ClientNode>(&'a self, id: u64) -> NodeResult<Vec<&'a T>>;
    fn load_collect_children_recursive<'a, T: ClientNode>(
        &'a self,
        id: u64,
    ) -> NodeResult<Vec<&'a T>>;
    fn load_collect_parents<'a, T: ClientNode>(&'a self, id: u64) -> NodeResult<Vec<&'a T>>;
    fn load_collect_parents_recursive<'a, T: ClientNode>(
        &'a self,
        id: u64,
    ) -> NodeResult<Vec<&'a T>>;
}

impl<'w> ClientContextExt for Context<WorldSource<'w>> {
    fn rng(&mut self) -> NodeResult<&mut ChaCha8Rng> {
        self.source_mut()
            .get_rng()
            .ok_or_else(|| NodeError::custom("RNG only available for BattleSimulation contexts"))
    }
    fn color(&self, ui: &mut Ui) -> Color32 {
        self.get_var(VarName::color)
            .get_color()
            .unwrap_or_else(|_| ui.visuals().weak_text_color())
    }
    fn load<'a, T: BevyComponent>(&'a self, id: u64) -> NodeResult<&'a T> {
        self.load_entity(self.entity(id)?)
    }
    fn load_entity<'a, T: BevyComponent>(&'a self, entity: Entity) -> NodeResult<&'a T> {
        let world = self.source().world()?;
        if let Some(component) = world.get::<T>(entity) {
            return Ok(component);
        } else {
            return Err(NodeError::load_error("Failed to get component from entity"));
        }
    }
    fn load_mut<'a, T: BevyComponent<Mutability = Mutable>>(
        &'a mut self,
        id: u64,
    ) -> NodeResult<Mut<'a, T>> {
        self.load_entity_mut(self.entity(id)?)
    }
    fn load_entity_mut<'a, T: BevyComponent<Mutability = Mutable>>(
        &'a mut self,
        entity: Entity,
    ) -> NodeResult<Mut<'a, T>> {
        let world = self.source_mut().world_mut()?;
        if let Some(component) = world.get_mut::<T>(entity) {
            return Ok(component);
        } else {
            return Err(NodeError::load_error("Failed to get component from entity"));
        }
    }

    fn load_many<'a, T>(&'a self, ids: &Vec<u64>) -> NodeResult<Vec<&'a T>>
    where
        T: 'static + BevyComponent,
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

    fn world<'a>(&'a self) -> NodeResult<&'a World> {
        self.source().world()
    }

    fn world_mut<'a>(&'a mut self) -> NodeResult<&'a mut World> {
        self.source_mut().world_mut()
    }

    fn battle<'a>(&'a self) -> NodeResult<&'a BattleSimulation> {
        self.source().battle()
    }

    fn battle_mut<'a>(&'a mut self) -> NodeResult<&'a mut BattleSimulation> {
        self.source_mut().battle_mut()
    }

    fn t(&self) -> NodeResult<f32> {
        Ok(self.source().battle()?.t)
    }

    fn t_mut(&mut self) -> NodeResult<&mut f32> {
        Ok(&mut self.source_mut().battle_mut()?.t)
    }

    fn id(&self, entity: Entity) -> NodeResult<u64> {
        let world = self.world()?;
        if let Some(map) = world.get_resource::<NodeEntityMap>() {
            map.get_id(entity)
                .ok_or(NodeError::id_not_found(entity.index(), entity.generation()))
        } else {
            Err(NodeError::context_error(anyhow::anyhow!(
                "NodeEntityMap resource not found"
            )))
        }
    }

    fn entity(&self, id: u64) -> NodeResult<Entity> {
        let world = self.world()?;
        if let Some(map) = world.get_resource::<NodeEntityMap>() {
            map.get_entity(id).ok_or(NodeError::entity_not_found(id))
        } else {
            Err(NodeError::context_error(anyhow::anyhow!(
                "NodeEntityMap resource not found"
            )))
        }
    }

    fn add_id_entity_link(&mut self, id: u64, entity: Entity) -> NodeResult<()> {
        let world = self.world_mut()?;
        if let Some(mut map) = world.get_resource_mut::<NodeEntityMap>() {
            map.insert(id, entity);
            Ok(())
        } else {
            Err(NodeError::context_error(anyhow::anyhow!(
                "NodeEntityMap resource not found"
            )))
        }
    }

    fn remove_id_entity_link(&mut self, id: u64) -> NodeResult<Entity> {
        let world = self.world_mut()?;
        if let Some(mut map) = world.get_resource_mut::<NodeEntityMap>() {
            map.remove_by_id(id).ok_or(NodeError::entity_not_found(id))
        } else {
            Err(NodeError::context_error(anyhow::anyhow!(
                "NodeEntityMap resource not found"
            )))
        }
    }

    fn add_link_entities(&mut self, parent: Entity, child: Entity) -> NodeResult<()> {
        self.add_link(self.id(parent)?, self.id(child)?)
    }

    fn despawn(&mut self, id: u64) -> NodeResult<()> {
        let mut ids = self.children_recursive(id)?;
        ids.push(id);
        let entities = ids
            .into_iter()
            .filter_map(|id| self.entity(id).ok())
            .unique()
            .collect_vec();
        let world = self.world_mut()?;
        for entity in entities {
            world.despawn(entity);
        }
        Ok(())
    }

    fn collect_children<'a, T: ClientNode>(&'a self, id: u64) -> NodeResult<Vec<&'a T>> {
        Ok(self
            .get_children_of_kind(id, T::kind_s())?
            .into_iter()
            .filter_map(|id| self.load(id).ok())
            .collect_vec())
    }

    fn owner_entity(&self) -> NodeResult<Entity> {
        self.entity(self.owner().to_not_found()?)
    }

    fn load_first_parent<'a, T: ClientNode>(&'a self, id: u64) -> NodeResult<&'a T> {
        let parent_id = self.first_parent(id, T::kind_s())?;
        self.load(parent_id)
    }

    fn load_first_parent_recursive<'a, T: ClientNode>(&'a self, id: u64) -> NodeResult<&'a T> {
        let parent_id = self.first_parent_recursive(id, T::kind_s())?;
        self.load(parent_id)
    }

    fn load_first_child<'a, T: ClientNode>(&'a self, id: u64) -> NodeResult<&'a T> {
        let child_id = self.first_child(id, T::kind_s())?;
        self.load(child_id)
    }

    fn load_first_child_recursive<'a, T: ClientNode>(&'a self, id: u64) -> NodeResult<&'a T> {
        let child_id = self.first_child_recursive(id, T::kind_s())?;
        self.load(child_id)
    }

    fn load_collect_children<'a, T: ClientNode>(&'a self, id: u64) -> NodeResult<Vec<&'a T>> {
        let child_ids = self.collect_kind_children(id, T::kind_s())?;
        self.load_many(&child_ids)
    }

    fn load_collect_children_recursive<'a, T: ClientNode>(
        &'a self,
        id: u64,
    ) -> NodeResult<Vec<&'a T>> {
        let child_ids = self.collect_kind_children_recursive(id, T::kind_s())?;
        self.load_many(&child_ids)
    }

    fn load_collect_parents<'a, T: ClientNode>(&'a self, id: u64) -> NodeResult<Vec<&'a T>> {
        let parent_ids = self.collect_kind_parents(id, T::kind_s())?;
        self.load_many(&parent_ids)
    }

    fn load_collect_parents_recursive<'a, T: ClientNode>(
        &'a self,
        id: u64,
    ) -> NodeResult<Vec<&'a T>> {
        let parent_ids = self.collect_kind_parents_recursive(id, T::kind_s())?;
        self.load_many(&parent_ids)
    }
}

/// Extension for using Context with Bevy World
pub trait WorldContextExt {
    /// Execute with a context using this world as the source (immutable)
    fn with_context<R, F>(&self, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut Context<WorldSource<'_>>) -> NodeResult<R>;

    /// Execute with a context using this world as the source (mutable)
    fn with_context_mut<R, F>(&mut self, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut Context<WorldSource<'_>>) -> NodeResult<R>;

    /// Execute with a context using this world as the source (immutable)
    fn as_context(&self) -> Context<WorldSource<'_>>;

    /// Execute with a context using this world as the source (mutable)
    fn as_context_mut(&mut self) -> Context<WorldSource<'_>>;
}

impl WorldContextExt for World {
    fn with_context<R, F>(&self, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut Context<WorldSource<'_>>) -> NodeResult<R>,
    {
        let source = WorldSource::new_immutable(self);
        Context::exec(source, f)
    }

    fn with_context_mut<R, F>(&mut self, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut Context<WorldSource<'_>>) -> NodeResult<R>,
    {
        let source = WorldSource::new_mutable(self);
        Context::exec(source, f)
    }

    fn as_context(&self) -> Context<WorldSource<'_>> {
        Context::new(WorldSource::new_immutable(self))
    }

    fn as_context_mut(&mut self) -> Context<WorldSource<'_>> {
        Context::new(WorldSource::new_mutable(self))
    }
}

impl WorldContextExt for BattleSimulation {
    fn as_context(&self) -> Context<WorldSource<'_>> {
        panic!()
    }

    fn as_context_mut(&mut self) -> Context<WorldSource<'_>> {
        Context::new(WorldSource::new_battle(self))
    }

    fn with_context<R, F>(&self, _f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut Context<WorldSource<'_>>) -> NodeResult<R>,
    {
        panic!()
    }

    fn with_context_mut<R, F>(&mut self, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut Context<WorldSource<'_>>) -> NodeResult<R>,
    {
        let source = WorldSource::new_battle(self);
        Context::exec(source, f)
    }
}

/// Type alias for convenience
pub type ClientContext<'w> = Context<WorldSource<'w>>;

pub const EMPTY_CONTEXT: ClientContext = Context::new(WorldSource::new_empty());

/// Extension trait for ClientContext to handle temporary layers
pub trait ClientContextTempLayers {
    /// Execute a closure with temporary layers added to a new ClientContext
    fn with_temp_layers<R, F>(&self, layers: Vec<ContextLayer>, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut ClientContext) -> NodeResult<R>;

    /// Execute a closure with a temporary layer added to a new ClientContext
    fn with_temp_layer<R, F>(&self, layer: ContextLayer, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut ClientContext) -> NodeResult<R>;

    /// Execute a closure with temporary owner layer
    fn with_temp_owner<R, F>(&self, owner_id: u64, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut ClientContext) -> NodeResult<R>;

    /// Execute a closure with temporary target layer
    fn with_temp_target<R, F>(&self, target_id: u64, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut ClientContext) -> NodeResult<R>;

    /// Execute a closure with temporary caster layer
    fn with_temp_caster<R, F>(&self, caster_id: u64, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut ClientContext) -> NodeResult<R>;

    /// Execute a closure with temporary variable layer
    fn with_temp_var<R, F>(&self, name: VarName, value: VarValue, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut ClientContext) -> NodeResult<R>;
}

/// Implementation of temporary layers for ClientContext
impl<'w> ClientContextTempLayers for ClientContext<'w> {
    fn with_temp_layers<R, F>(&self, layers: Vec<ContextLayer>, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut ClientContext) -> NodeResult<R>,
    {
        let world = self.source().world()?;
        let mut temp_context = Context::with_layers(WorldSource::new_immutable(world), layers);
        f(&mut temp_context)
    }

    fn with_temp_layer<R, F>(&self, layer: ContextLayer, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut ClientContext) -> NodeResult<R>,
    {
        self.with_temp_layers(vec![layer], f)
    }

    fn with_temp_owner<R, F>(&self, owner_id: u64, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut ClientContext) -> NodeResult<R>,
    {
        self.with_temp_layer(ContextLayer::Owner(owner_id), f)
    }

    fn with_temp_target<R, F>(&self, target_id: u64, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut ClientContext) -> NodeResult<R>,
    {
        self.with_temp_layer(ContextLayer::Target(target_id), f)
    }

    fn with_temp_caster<R, F>(&self, caster_id: u64, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut ClientContext) -> NodeResult<R>,
    {
        self.with_temp_layer(ContextLayer::Caster(caster_id), f)
    }

    fn with_temp_var<R, F>(&self, name: VarName, value: VarValue, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut ClientContext) -> NodeResult<R>,
    {
        self.with_temp_layer(ContextLayer::Var(name, value), f)
    }
}
