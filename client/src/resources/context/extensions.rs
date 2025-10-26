use super::{client_source::*, node_maps::*};
use crate::prelude::*;
use crate::resources::battle::BattleSimulation;
use rand_chacha::ChaCha8Rng;

// Extension trait for Context to load nodes in client
pub trait ClientContextExt {
    fn is_battle(&self) -> bool;
    fn rng(&mut self) -> NodeResult<&mut ChaCha8Rng>;
    fn color(&self, ui: &mut Ui) -> Color32;
    fn load<T: ClientNode + Clone>(&self, id: u64) -> NodeResult<T>;
    fn load_ref<'a, T: ClientNode>(&'a self, id: u64) -> NodeResult<&'a T>;
    fn load_entity<'a, T: BevyComponent>(&'a self, entity: Entity) -> NodeResult<&'a T>;
    fn load_mut<'a, T: BevyComponent<Mutability = bevy::ecs::component::Mutable>>(
        &'a mut self,
        id: u64,
    ) -> NodeResult<Mut<'a, T>>;
    fn load_many<T: ClientNode + Clone>(&self, ids: &Vec<u64>) -> NodeResult<Vec<T>>;
    fn load_many_ref<'a, T: ClientNode>(&'a self, ids: &Vec<u64>) -> NodeResult<Vec<&'a T>>;
    fn load_children<T: ClientNode + Clone>(&self, from_id: u64) -> NodeResult<Vec<T>>;
    fn load_children_ref<'a, T: ClientNode>(&'a self, from_id: u64) -> NodeResult<Vec<&'a T>>;
    fn world<'a>(&'a self) -> NodeResult<&'a World>;
    fn world_mut<'a>(&'a mut self) -> NodeResult<&'a mut World>;
    fn battle<'a>(&'a self) -> NodeResult<&'a BattleSimulation>;
    fn battle_mut<'a>(&'a mut self) -> NodeResult<&'a mut BattleSimulation>;
    fn t(&self) -> NodeResult<f32>;
    fn t_mut(&mut self) -> NodeResult<&mut f32>;
    fn entity(&self, id: u64) -> NodeResult<Entity>;
    fn ids(&self, entity: Entity) -> NodeResult<HashSet<u64>>;
    fn add_id_entity_link(&mut self, id: u64, entity: Entity) -> NodeResult<()>;

    fn despawn(&mut self, id: u64) -> NodeResult<()>;
    fn collect_children<'a, T: ClientNode>(&'a self, id: u64) -> NodeResult<Vec<&'a T>>;
    fn owner_entity(&self) -> NodeResult<Entity>;
    fn load_first_parent<T: ClientNode + Clone>(&self, id: u64) -> NodeResult<T>;
    fn load_first_parent_recursive<T: ClientNode + Clone>(&self, id: u64) -> NodeResult<T>;
    fn load_collect_children<T: ClientNode + Clone>(&self, id: u64) -> NodeResult<Vec<T>>;

    // Layer methods for context scoping
    fn with_layers_ref<R, F>(&self, layers: impl Into<Vec<ContextLayer>>, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut super::ClientContext) -> NodeResult<R>;
    fn with_owner_ref<R, F>(&self, owner_id: u64, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut super::ClientContext) -> NodeResult<R>;
}

impl<'w> ClientContextExt for Context<ClientSource<'w>> {
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

    fn load<T: ClientNode + Clone>(&self, id: u64) -> NodeResult<T> {
        match self.source() {
            ClientSource::Source(source) | ClientSource::SourceMut(source) => {
                let source_guard = source.read().unwrap();
                let world_guard = source_guard.world();
                let world = world_guard.read().unwrap();
                let node_map_guard = source_guard.node_map();
                let node_map = node_map_guard.read().unwrap();

                if let Some(entity) = node_map.get_entity(id) {
                    world.get::<T>(entity).cloned().to_not_found_id(id)
                } else {
                    Err(NodeError::entity_not_found(id))
                }
            }
            ClientSource::WorldRef(..)
            | ClientSource::WorldMut(..)
            | ClientSource::BattleMut(..)
            | ClientSource::BattleRef(..) => {
                let entity = self.entity(id)?;
                let world = self.world()?;
                world.get::<T>(entity).cloned().to_not_found_id(id)
            }
            ClientSource::Db(_) => cn()
                .db
                .nodes_world()
                .id()
                .find(&id)
                .to_not_found_id(id)?
                .to_node::<T>(),
            ClientSource::None => Err(NodeError::custom("Cannot load from None source")),
        }
    }

    fn load_ref<'a, T: ClientNode>(&'a self, id: u64) -> NodeResult<&'a T> {
        match self.source() {
            ClientSource::Source(_source) | ClientSource::SourceMut(_source) => {
                // For source contexts, we need to be careful with lifetimes
                // This is a limitation - we can't return references through the locks
                Err(NodeError::custom(
                    "Cannot get reference from Source context, use load() instead",
                ))
            }
            ClientSource::WorldRef(..)
            | ClientSource::WorldMut(..)
            | ClientSource::BattleMut(..)
            | ClientSource::BattleRef(..) => {
                let entity = self.entity(id)?;
                let world = self.world()?;
                world.get::<T>(entity).to_not_found_id(id)
            }
            ClientSource::Db(_) => Err(NodeError::custom(
                "Cannot get reference from Db source, use load() instead",
            )),
            _ => Err(NodeError::custom("Cannot load from this source")),
        }
    }

    fn load_entity<'a, T: BevyComponent>(&'a self, entity: Entity) -> NodeResult<&'a T> {
        let world = self.source().world()?;
        world.get::<T>(entity).to_not_found()
    }

    fn load_mut<'a, T: BevyComponent<Mutability = bevy::ecs::component::Mutable>>(
        &'a mut self,
        id: u64,
    ) -> NodeResult<Mut<'a, T>> {
        let entity = self.entity(id)?;
        let world = self.world_mut()?;
        world.get_mut::<T>(entity).to_not_found()
    }

    fn load_many<T>(&self, ids: &Vec<u64>) -> NodeResult<Vec<T>>
    where
        T: ClientNode + Clone,
    {
        let mut result = Vec::new();
        for id in ids {
            result.push(self.load::<T>(*id)?);
        }
        Ok(result)
    }

    fn load_many_ref<'a, T>(&'a self, ids: &Vec<u64>) -> NodeResult<Vec<&'a T>>
    where
        T: ClientNode,
    {
        let mut result = Vec::new();
        for id in ids {
            result.push(self.load_ref::<T>(*id)?);
        }
        Ok(result)
    }

    fn load_children<T: ClientNode + Clone>(&self, from_id: u64) -> NodeResult<Vec<T>> {
        let kind = T::kind_s();
        let ids = self.get_children_of_kind(from_id, kind)?;
        self.load_many(&ids)
    }

    fn load_children_ref<'a, T: ClientNode>(&'a self, from_id: u64) -> NodeResult<Vec<&'a T>> {
        let kind = T::kind_s();
        let ids = self.get_children_of_kind(from_id, kind)?;
        self.load_many_ref(&ids)
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

    fn entity(&self, id: u64) -> NodeResult<Entity> {
        match self.source() {
            ClientSource::Source(source) | ClientSource::SourceMut(source) => {
                let source_guard = source.read().unwrap();
                let node_map_guard = source_guard.node_map();
                let node_map = node_map_guard.read().unwrap();
                node_map
                    .get_entity(id)
                    .ok_or(NodeError::entity_not_found(id))
            }
            ClientSource::WorldRef(world) => {
                if let Some(map) = world.get_resource::<SmartNodeMap>() {
                    map.get_entity(id).ok_or(NodeError::entity_not_found(id))
                } else {
                    Err(NodeError::custom("SmartNodeMap resource not found"))
                }
            }
            ClientSource::WorldMut(world) => {
                if let Some(map) = world.get_resource::<SmartNodeMap>() {
                    map.get_entity(id).ok_or(NodeError::entity_not_found(id))
                } else {
                    Err(NodeError::custom("SmartNodeMap resource not found"))
                }
            }
            ClientSource::BattleMut(battle, _) => {
                if let Some(map) = battle.world.get_resource::<SmartNodeMap>() {
                    map.get_entity(id).ok_or(NodeError::entity_not_found(id))
                } else {
                    Err(NodeError::custom("SmartNodeMap resource not found"))
                }
            }
            ClientSource::BattleRef(battle, _) => {
                if let Some(map) = battle.world.get_resource::<SmartNodeMap>() {
                    map.get_entity(id).ok_or(NodeError::entity_not_found(id))
                } else {
                    Err(NodeError::custom("SmartNodeMap resource not found"))
                }
            }
            _ => Err(NodeError::custom("Cannot get entity from this source")),
        }
    }

    fn ids(&self, entity: Entity) -> NodeResult<HashSet<u64>> {
        match self.source() {
            ClientSource::Source(source) | ClientSource::SourceMut(source) => {
                let source_guard = source.read().unwrap();
                let node_map_guard = source_guard.node_map();
                let node_map = node_map_guard.read().unwrap();
                let ids = node_map.get_node_ids(entity);
                if ids.is_empty() {
                    Err(NodeError::entity_not_found(entity.index() as u64))
                } else {
                    Ok(ids.into_iter().collect())
                }
            }
            _ => {
                let world = self.world()?;
                if let Some(map) = world.get_resource::<SmartNodeMap>() {
                    let ids = map.get_node_ids(entity);
                    if ids.is_empty() {
                        Err(NodeError::entity_not_found(entity.index() as u64))
                    } else {
                        Ok(ids.into_iter().collect())
                    }
                } else {
                    Err(NodeError::custom("SmartNodeMap resource not found"))
                }
            }
        }
    }

    fn add_id_entity_link(&mut self, id: u64, entity: Entity) -> NodeResult<()> {
        match self.source_mut() {
            ClientSource::SourceMut(_source) => {
                // For source contexts, entities are managed automatically
                // This is mainly used for legacy compatibility
                Ok(())
            }
            _ => {
                let world = self.world_mut()?;
                if let Some(mut map) = world.get_resource_mut::<SmartNodeMap>() {
                    // This is a simplified approach for legacy compatibility
                    // In practice, the smart node map should handle this automatically
                    Ok(())
                } else {
                    let map = SmartNodeMap::new();
                    world.insert_resource(map);
                    Ok(())
                }
            }
        }
    }

    fn despawn(&mut self, id: u64) -> NodeResult<()> {
        match self.source_mut() {
            ClientSource::SourceMut(source) => {
                let mut source_guard = source.write().unwrap();
                source_guard.delete_node(id);
                Ok(())
            }
            _ => {
                let entity = self.entity(id)?;
                let world = self.world_mut()?;
                world.despawn(entity);
                Ok(())
            }
        }
    }

    fn collect_children<'a, T: ClientNode>(&'a self, id: u64) -> NodeResult<Vec<&'a T>> {
        let kind = T::kind_s();
        let ids = self.get_children_of_kind(id, kind)?;
        Ok(ids
            .into_iter()
            .filter_map(|child_id| self.load_ref::<T>(child_id).ok())
            .collect::<Vec<_>>())
    }

    fn owner_entity(&self) -> NodeResult<Entity> {
        self.entity(self.owner().to_not_found()?)
    }

    fn load_first_parent<T: ClientNode + Clone>(&self, id: u64) -> NodeResult<T> {
        let parent_id = self.first_parent(id, T::kind_s())?;
        self.load(parent_id)
    }

    fn load_first_parent_recursive<T: ClientNode + Clone>(&self, id: u64) -> NodeResult<T> {
        let parent_id = self.first_parent_recursive(id, T::kind_s())?;
        self.load(parent_id)
    }

    fn load_collect_children<T: ClientNode + Clone>(&self, id: u64) -> NodeResult<Vec<T>> {
        let child_ids = self.collect_kind_children(id, T::kind_s())?;
        self.load_many(&child_ids)
    }

    fn with_layers_ref<R, F>(&self, layers: impl Into<Vec<ContextLayer>>, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut super::ClientContext) -> NodeResult<R>,
    {
        let mut merged_layers = self.layers().clone();
        merged_layers.append(&mut layers.into());

        let mut temp_ctx = if let Ok(battle) = self.battle() {
            let t = self.t().unwrap_or(0.0);
            Context::new_with_layers(ClientSource::new_battle(battle, t), merged_layers)
        } else {
            let world = self.world()?;
            Context::new_with_layers(ClientSource::new_immutable(world), merged_layers)
        };
        f(&mut temp_ctx)
    }

    fn with_owner_ref<R, F>(&self, owner_id: u64, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut super::ClientContext) -> NodeResult<R>,
    {
        self.with_layers_ref(vec![ContextLayer::Owner(owner_id)], f)
    }
}

// Extension for using Context with Bevy World
pub trait WorldContextExt {
    fn with_context<R, F>(&self, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut super::ClientContext<'_>) -> NodeResult<R>;
    fn with_context_mut<R, F>(&mut self, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut super::ClientContext<'_>) -> NodeResult<R>;
    fn as_context(&self) -> super::ClientContext<'_>;
    fn as_context_mut(&mut self) -> super::ClientContext<'_>;
}

impl WorldContextExt for World {
    fn with_context<R, F>(&self, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut super::ClientContext<'_>) -> NodeResult<R>,
    {
        let source = ClientSource::new_immutable(self);
        Context::exec(source, f)
    }

    fn with_context_mut<R, F>(&mut self, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut super::ClientContext<'_>) -> NodeResult<R>,
    {
        let source = ClientSource::new_mutable(self);
        Context::exec(source, f)
    }

    fn as_context(&self) -> super::ClientContext<'_> {
        Context::new(ClientSource::new_immutable(self))
    }

    fn as_context_mut(&mut self) -> super::ClientContext<'_> {
        Context::new(ClientSource::new_mutable(self))
    }
}

// Re-export NodeEntity for compatibility
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

    pub fn add_node(&mut self, id: u64, kind: NodeKind) {
        self.nodes.insert(kind, id);
    }

    pub fn get_kind(&self, id: u64) -> Option<NodeKind> {
        self.nodes.iter().find(|(_, v)| **v == id).map(|(k, _)| *k)
    }

    pub fn ids(&self) -> HashSet<u64> {
        self.nodes.values().copied().collect()
    }

    pub fn get_node_ids(&self) -> Vec<u64> {
        self.nodes.values().copied().collect()
    }
}
