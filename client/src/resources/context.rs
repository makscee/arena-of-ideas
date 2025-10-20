use super::*;
use bevy::ecs::component::Mutable;
use schema::{Context, ContextSource, NodeError, NodeResult};
use std::collections::HashMap;
use utils_client::node_kind_match;

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

    pub fn add_link(&mut self, id: u64, entity: Entity) {
        self.insert(id, entity);
    }

    pub fn remove_link(&mut self, id: u64) -> Option<Entity> {
        self.remove_by_id(id)
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

    pub fn remove_all_links(&mut self, node_id: u64) {
        // Remove all outgoing links from this node
        self.links.remove(&node_id);

        // Remove all incoming links to this node
        for (_, links) in self.links.iter_mut() {
            links.retain(|(id, _)| *id != node_id);
        }

        // Remove from reverse links
        self.reverse_links.remove(&node_id);

        // Remove from all parent lists in reverse links
        for (_, parents) in self.reverse_links.iter_mut() {
            parents.retain(|(id, _)| *id != node_id);
        }
    }

    pub fn clear(&mut self) {
        self.links.clear();
        self.reverse_links.clear();
    }
}

/// Marker component for entities with nodes
#[derive(BevyComponent, Debug)]
pub struct NodeEntity {
    pub nodes: HashMap<NodeKind, u64>,
}

impl NodeEntity {
    pub fn new(id: u64, kind: NodeKind) -> Self {
        Self {
            nodes: HashMap::from_iter(Some((kind, id))),
        }
    }

    pub fn with_nodes(nodes: impl IntoIterator<Item = (NodeKind, u64)>) -> Self {
        Self {
            nodes: HashMap::from_iter(nodes),
        }
    }

    pub fn add_node(&mut self, id: u64, kind: NodeKind) {
        self.nodes.insert(kind, id);
    }

    pub fn get_node_kinds(&self) -> Vec<NodeKind> {
        self.nodes.keys().copied().collect()
    }

    pub fn get_node_ids(&self) -> Vec<u64> {
        self.nodes.values().copied().collect_vec()
    }

    pub fn has_kind(&self, kind: NodeKind) -> bool {
        self.nodes.contains_key(&kind)
    }

    pub fn get_kind(&self, id: u64) -> Option<NodeKind> {
        self.nodes.iter().find(|(_, v)| **v == id).map(|(k, _)| *k)
    }
}

/// Unified WorldSource enum for both immutable and mutable World access
pub enum WorldSource<'w> {
    WorldRef(&'w World),
    WorldMut(&'w mut World),
    BattleMut(&'w mut BattleSimulation, f32),
    BattleRef(&'w BattleSimulation, f32),
    None,
}

impl<'w> WorldSource<'w> {
    pub fn is_battle(&self) -> bool {
        matches!(
            self,
            WorldSource::BattleMut(..) | WorldSource::BattleRef(..)
        )
    }

    pub fn new_immutable(world: &'w World) -> Self {
        Self::WorldRef(world)
    }

    pub fn new_mutable(world: &'w mut World) -> Self {
        Self::WorldMut(world)
    }

    pub const fn new_empty() -> Self {
        Self::None
    }

    pub fn new_battle_mut(battle: &'w mut BattleSimulation, t: f32) -> Self {
        Self::BattleMut(battle, t)
    }

    pub fn new_battle(battle: &'w BattleSimulation, t: f32) -> Self {
        Self::BattleRef(battle, t)
    }

    pub fn battle(&self) -> NodeResult<&BattleSimulation> {
        match self {
            Self::BattleMut(battle, _) => Ok(battle),
            Self::BattleRef(battle, _) => Ok(battle),
            _ => Err(NodeError::custom("Source is not a BattleSimulation")),
        }
    }

    pub fn battle_mut(&mut self) -> NodeResult<&mut BattleSimulation> {
        match self {
            Self::BattleMut(battle, _) => Ok(battle),
            _ => Err(NodeError::custom("Source is not a BattleSimulation")),
        }
    }

    pub fn battle_t(&self) -> NodeResult<f32> {
        match self {
            Self::BattleMut(_, t) => Ok(*t),
            Self::BattleRef(_, t) => Ok(*t),
            _ => Err(NodeError::custom("Source is not a BattleSimulation")),
        }
    }

    pub fn battle_t_mut(&mut self) -> NodeResult<&mut f32> {
        match self {
            Self::BattleMut(_, t) => Ok(t),
            _ => Err(NodeError::custom("Source is not a BattleSimulation")),
        }
    }

    fn get_rng(&mut self) -> Option<&mut ChaCha8Rng> {
        match self {
            Self::BattleMut(battle, _) => Some(&mut battle.rng),
            _ => None,
        }
    }

    pub fn world(&self) -> NodeResult<&World> {
        match self {
            Self::WorldRef(world) => Ok(world),
            Self::WorldMut(world) => Ok(world),
            Self::BattleMut(battle, _) => Ok(&battle.world),
            Self::BattleRef(battle, _) => Ok(&battle.world),
            Self::None => Err(NodeError::custom("Source is None")),
        }
    }

    pub fn world_mut(&mut self) -> NodeResult<&mut World> {
        match self {
            Self::WorldMut(world) => Ok(world),
            Self::BattleMut(battle, _) => Ok(&mut battle.world),
            Self::WorldRef(_) => Err(NodeError::custom("Source World is immutable")),
            Self::BattleRef(_, _) => Err(NodeError::custom("Source World is immutable")),
            Self::None => Err(NodeError::custom("Source World not set")),
        }
    }
}

impl<'w> ContextSource for WorldSource<'w> {
    fn get_node_kind(&self, id: u64) -> NodeResult<NodeKind> {
        let world = self.world()?;
        let entity = world
            .resource::<NodeEntityMap>()
            .get_entity(id)
            .to_not_found()?;
        world
            .get::<NodeEntity>(entity)
            .to_not_found()?
            .get_kind(id)
            .to_not_found()
    }

    fn get_children(&self, id: u64) -> NodeResult<Vec<u64>> {
        Ok(self.world()?.resource::<NodeLinks>().get_children(id))
    }

    fn get_children_of_kind(&self, from_id: u64, kind: NodeKind) -> NodeResult<Vec<u64>> {
        let world = self.world()?;
        Ok(world
            .resource::<NodeLinks>()
            .get_children_of_kind(from_id, kind))
    }

    fn get_parents(&self, id: u64) -> NodeResult<Vec<u64>> {
        Ok(self.world()?.resource::<NodeLinks>().get_parents(id))
    }

    fn get_parents_of_kind(&self, id: u64, kind: NodeKind) -> NodeResult<Vec<u64>> {
        Ok(self
            .world()?
            .resource::<NodeLinks>()
            .get_parents_of_kind(id, kind))
    }

    fn add_link(&mut self, from_id: u64, to_id: u64) -> NodeResult<()> {
        let to_kind = self.get_node_kind(to_id).track()?;
        self.world_mut()?
            .resource_mut::<NodeLinks>()
            .add_link(from_id, to_id, to_kind);
        Ok(())
    }

    fn remove_link(&mut self, from_id: u64, to_id: u64) -> NodeResult<()> {
        self.world_mut()?
            .resource_mut::<NodeLinks>()
            .remove_link(from_id, to_id);
        Ok(())
    }

    fn is_linked(&self, from_id: u64, to_id: u64) -> NodeResult<bool> {
        let world = self.world()?;
        if let Some(links) = world.get_resource::<NodeLinks>() {
            Ok(links.has_link(from_id, to_id))
        } else {
            Ok(false)
        }
    }

    fn insert_node(&mut self, id: u64, owner: u64, kind: NodeKind, data: String) -> NodeResult<()> {
        let world = self.world_mut()?;
        let mut nem = world.remove_resource::<NodeEntityMap>().to_not_found()?;
        let entity = nem
            .get_entity(id)
            .unwrap_or_else(|| world.spawn_empty().id());
        nem.add_link(id, entity);
        world.insert_resource(nem);
        if let Some(mut ne) = world.get_mut::<NodeEntity>(entity) {
            ne.add_node(id, kind);
        } else {
            world.entity_mut(entity).insert(NodeEntity::new(id, kind));
        }
        node_kind_match!(kind, {
            let mut n = NodeType::default();
            n.inject_data(&data)?;
            n.set_id(id);
            n.set_owner(owner);
            world.entity_mut(entity).insert(n);
        });

        Ok(())
    }

    fn delete_node(&mut self, id: u64) -> NodeResult<()> {
        let world = self.world_mut()?;

        // Get the entity for this node
        let entity = {
            if let Some(map) = world.get_resource::<NodeEntityMap>() {
                if let Some(entity) = map.get_entity(id) {
                    entity
                } else {
                    return Err(NodeError::custom(format!("Entity not found for id {}", id)));
                }
            } else {
                return Err(NodeError::custom("NodeEntityMap resource not found"));
            }
        };
        let node_entity = world.get::<NodeEntity>(entity).to_not_found()?;
        let entity_ids = node_entity.get_node_ids();
        if entity_ids.len() == 1 {
            if entity_ids[0] == id {
                world.entity_mut(entity).despawn();
                return Ok(());
            } else {
                panic!();
            }
        }
        let kind = node_entity.get_kind(id).to_not_found()?;
        node_kind_match!(kind, {
            world.entity_mut(entity).remove::<NodeType>();
        });
        Ok(())
    }

    fn set_var(&mut self, id: u64, var: VarName, value: VarValue) -> NodeResult<()> {
        // For battle simulations, also track in NodeStateHistory
        if let WorldSource::BattleMut(_battle, t) = self {
            let t = *t;
            let world = self.world_mut().track()?;
            if let Some(map) = world.get_resource::<NodeEntityMap>() {
                if let Some(entity) = map.get_entity(id) {
                    if let Some(mut node_state_history) = world.get_mut::<NodeStateHistory>(entity)
                    {
                        node_state_history.insert(t, 0.0, var, value.clone());
                    } else {
                        let mut node_state_history = NodeStateHistory::default();
                        node_state_history.insert(t, 0.0, var, value.clone());
                        world.entity_mut(entity).insert(node_state_history);
                    }
                } else {
                    return Err(NodeError::entity_not_found(id));
                }
            } else {
                return Err(NodeError::not_found_generic("NodeEntityMap not found"));
            }
        }
        Ok(())
    }

    fn get_var_direct(&self, id: u64, var: VarName) -> NodeResult<VarValue> {
        if self.battle().is_ok() {
            let t = self.battle_t()?;
            let world = self.world()?;
            if let Some(map) = world.get_resource::<NodeEntityMap>() {
                if let Some(entity) = map.get_entity(id) {
                    if let Some(node_state_history) = world.get::<NodeStateHistory>(entity) {
                        if let Some(value) = node_state_history.get_at(t, var) {
                            return Ok(value);
                        }
                    }
                }
            }
        }

        // For global world or if not found in history, get from the node directly
        let kind = self.get_node_kind(id)?;
        node_kind_match!(kind, {
            let node: NodeType = {
                let world = self.world()?;
                let entity = world
                    .resource::<NodeEntityMap>()
                    .get_entity(id)
                    .to_not_found()?;
                world.get::<NodeType>(entity).to_not_found()?.clone()
            };
            node.get_var(var)
        })
    }
}

impl<'w> ContextSource for &mut WorldSource<'w> {
    fn get_node_kind(&self, id: u64) -> NodeResult<NodeKind> {
        (**self).get_node_kind(id)
    }

    fn get_children(&self, from_id: u64) -> NodeResult<Vec<u64>> {
        (**self).get_children(from_id)
    }

    fn get_children_of_kind(&self, from_id: u64, kind: NodeKind) -> NodeResult<Vec<u64>> {
        (**self).get_children_of_kind(from_id, kind)
    }

    fn get_parents(&self, from_id: u64) -> NodeResult<Vec<u64>> {
        (**self).get_parents(from_id)
    }

    fn get_parents_of_kind(&self, from_id: u64, kind: NodeKind) -> NodeResult<Vec<u64>> {
        (**self).get_parents_of_kind(from_id, kind)
    }

    fn add_link(&mut self, from_id: u64, to_id: u64) -> NodeResult<()> {
        (**self).add_link(from_id, to_id)
    }

    fn remove_link(&mut self, from_id: u64, to_id: u64) -> NodeResult<()> {
        (**self).remove_link(from_id, to_id)
    }

    fn is_linked(&self, from_id: u64, to_id: u64) -> NodeResult<bool> {
        (**self).is_linked(from_id, to_id)
    }

    fn insert_node(&mut self, id: u64, owner: u64, kind: NodeKind, data: String) -> NodeResult<()> {
        (**self).insert_node(id, owner, kind, data)
    }

    fn delete_node(&mut self, id: u64) -> NodeResult<()> {
        (**self).delete_node(id)
    }

    fn get_var_direct(&self, id: u64, var: VarName) -> NodeResult<VarValue> {
        (**self).get_var_direct(id, var)
    }

    fn set_var(&mut self, id: u64, var: VarName, value: VarValue) -> NodeResult<()> {
        (**self).set_var(id, var, value)
    }
}

/// Extension trait for Context to load nodes in client
pub trait ClientContextExt {
    fn is_battle(&self) -> bool;
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
    fn is_battle(&self) -> bool {
        self.source().is_battle()
    }

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
        let world = self.source_mut().world_mut().track()?;
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
        self.source().battle_t()
    }

    fn t_mut(&mut self) -> NodeResult<&mut f32> {
        self.source_mut().battle_t_mut()
    }

    fn id(&self, entity: Entity) -> NodeResult<u64> {
        let world = self.world()?;
        if let Some(map) = world.get_resource::<NodeEntityMap>() {
            map.get_id(entity).ok_or(NodeError::id_not_found(
                entity.index(),
                entity.generation().to_bits(),
            ))
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
        self.add_link(self.id(parent)?, self.id(child)?).track()
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

/// Type alias for convenience
pub type ClientContext<'w> = Context<WorldSource<'w>>;

pub const EMPTY_CONTEXT: ClientContext = Context::new(WorldSource::new_empty());

/// Extension trait for ClientContext to handle temporary layers
pub trait ClientContextLayersRef {
    /// Execute a closure with temporary layers added to a new ClientContext
    fn with_layers_ref<R, F>(&self, layers: impl Into<Vec<ContextLayer>>, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut ClientContext) -> NodeResult<R>;

    /// Execute a closure with a temporary layer added to a new ClientContext
    fn with_layer_ref<R, F>(&self, layer: ContextLayer, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut ClientContext) -> NodeResult<R>;

    /// Execute a closure with temporary owner layer
    fn with_owner_ref<R, F>(&self, owner_id: u64, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut ClientContext) -> NodeResult<R>;

    /// Execute a closure with temporary target layer
    fn with_target_ref<R, F>(&self, target_id: u64, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut ClientContext) -> NodeResult<R>;

    /// Execute a closure with temporary caster layer
    fn with_caster_ref<R, F>(&self, caster_id: u64, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut ClientContext) -> NodeResult<R>;

    /// Execute a closure with temporary caster layer
    fn with_status_ref<R, F>(&self, status_id: u64, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut ClientContext) -> NodeResult<R>;

    /// Execute a closure with temporary variable layer
    fn with_var_ref<R, F>(&self, name: VarName, value: VarValue, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut ClientContext) -> NodeResult<R>;

    fn scope_ref<R, F>(&self, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut ClientContext) -> NodeResult<R>;
}

/// Implementation of temporary layers for ClientContext
impl<'w> ClientContextLayersRef for ClientContext<'w> {
    fn with_layers_ref<R, F>(&self, layers: impl Into<Vec<ContextLayer>>, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut ClientContext) -> NodeResult<R>,
    {
        let mut merged_layers = self.layers().clone();
        merged_layers.append(&mut layers.into());
        let mut temp_ctx = if let Ok(battle) = self.source().battle() {
            let t = self.source().battle_t().unwrap_or(0.0);
            Context::new_with_layers(WorldSource::new_battle(battle, t), merged_layers)
        } else {
            let world = self.source().world()?;
            Context::new_with_layers(WorldSource::new_immutable(world), merged_layers)
        };
        f(&mut temp_ctx)
    }

    fn with_layer_ref<R, F>(&self, layer: ContextLayer, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut ClientContext) -> NodeResult<R>,
    {
        self.with_layers_ref(vec![layer], f)
    }

    fn with_owner_ref<R, F>(&self, owner_id: u64, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut ClientContext) -> NodeResult<R>,
    {
        self.with_layer_ref(ContextLayer::Owner(owner_id), f)
    }

    fn with_target_ref<R, F>(&self, target_id: u64, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut ClientContext) -> NodeResult<R>,
    {
        self.with_layer_ref(ContextLayer::Target(target_id), f)
    }

    fn with_caster_ref<R, F>(&self, caster_id: u64, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut ClientContext) -> NodeResult<R>,
    {
        self.with_layer_ref(ContextLayer::Caster(caster_id), f)
    }

    fn with_status_ref<R, F>(&self, status_id: u64, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut ClientContext) -> NodeResult<R>,
    {
        self.with_layer_ref(ContextLayer::Status(status_id), f)
    }

    fn with_var_ref<R, F>(&self, name: VarName, value: VarValue, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut ClientContext) -> NodeResult<R>,
    {
        self.with_layer_ref(ContextLayer::Var(name, value), f)
    }

    fn scope_ref<R, F>(&self, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut ClientContext) -> NodeResult<R>,
    {
        self.with_layers_ref(Vec::new(), f)
    }
}
