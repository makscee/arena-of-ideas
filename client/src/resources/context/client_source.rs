use super::{node_maps::*, sources::*};
use crate::prelude::*;
use crate::resources::battle::BattleSimulation;
use crate::stdb::RemoteTables;
use rand_chacha::ChaCha8Rng;
use std::sync::{Arc, RwLock};

// ClientSource enum for different contexts
pub enum ClientSource<'w> {
    /// Access to a specific node source (read-only)
    Source(Arc<RwLock<NodeSource>>),
    /// Access to a specific node source (mutable)
    SourceMut(Arc<RwLock<NodeSource>>),
    /// Direct world reference (legacy)
    WorldRef(&'w World),
    /// Direct world mutable reference (legacy)
    WorldMut(&'w mut World),
    /// Battle simulation context (mutable)
    BattleMut(&'w mut BattleSimulation, f32),
    /// Battle simulation context (read-only)
    BattleRef(&'w BattleSimulation, f32),
    /// Database source for exploration
    Db(Box<crate::plugins::explorer::DbSource<'w>>),
    /// Empty source
    None,
}

impl<'w> ClientSource<'w> {
    pub fn is_battle(&self) -> bool {
        matches!(
            self,
            ClientSource::BattleMut(..) | ClientSource::BattleRef(..)
        )
    }

    pub fn is_db(&self) -> bool {
        matches!(self, ClientSource::Db(_))
    }

    pub fn is_source(&self) -> bool {
        matches!(self, ClientSource::Source(_) | ClientSource::SourceMut(_))
    }

    pub fn new_source(source: Arc<RwLock<NodeSource>>) -> Self {
        Self::Source(source)
    }

    pub fn new_source_mut(source: Arc<RwLock<NodeSource>>) -> Self {
        Self::SourceMut(source)
    }

    pub fn new_solid() -> Self {
        Self::Source(crate::resources::context::node_sources().solid())
    }

    pub fn new_solid_mut() -> Self {
        Self::SourceMut(crate::resources::context::node_sources().solid())
    }

    pub fn new_top() -> Self {
        Self::Source(crate::resources::context::node_sources().top())
    }

    pub fn new_top_mut() -> Self {
        Self::SourceMut(crate::resources::context::node_sources().top())
    }

    pub fn new_selected() -> Self {
        Self::Source(crate::resources::context::node_sources().selected())
    }

    pub fn new_selected_mut() -> Self {
        Self::SourceMut(crate::resources::context::node_sources().selected())
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

    pub fn new_db(
        db: &'w RemoteTables,
        strategy: crate::plugins::explorer::DbLinkStrategy,
    ) -> Self {
        Self::Db(Box::new(crate::plugins::explorer::DbSource::with_strategy(
            db, strategy,
        )))
    }

    pub fn world(&self) -> NodeResult<&World> {
        match self {
            Self::Source(source) => {
                let world_guard = source.read().unwrap().world();
                let world_ref = world_guard.read().unwrap();
                unsafe { Ok(&*((&*world_ref) as *const World)) }
            }
            Self::SourceMut(source) => {
                let world_guard = source.read().unwrap().world();
                let world_ref = world_guard.read().unwrap();
                unsafe { Ok(&*((&*world_ref) as *const World)) }
            }
            Self::WorldRef(world) => Ok(world),
            Self::WorldMut(world) => Ok(world),
            Self::BattleMut(battle, _) => Ok(&battle.world),
            Self::BattleRef(battle, _) => Ok(&battle.world),
            Self::Db(_) => Err(NodeError::custom("Tried to get &World from Db context")),
            Self::None => Err(NodeError::custom("Source is None")),
        }
    }

    pub fn world_mut(&mut self) -> NodeResult<&mut World> {
        match self {
            Self::SourceMut(source) => {
                let world_guard = source.write().unwrap().world();
                let mut world_ref = world_guard.write().unwrap();
                unsafe { Ok(&mut *((&mut *world_ref) as *mut World)) }
            }
            Self::WorldMut(world) => Ok(world),
            Self::BattleMut(battle, _) => Ok(&mut battle.world),
            Self::WorldRef(_) => Err(NodeError::custom("Source World is immutable")),
            Self::BattleRef(_, _) => Err(NodeError::custom("Source World is immutable")),
            Self::Db(_) => Err(NodeError::custom("Tried to get &mut World from Db context")),
            Self::None => Err(NodeError::custom("Source World not set")),
            Self::Source(_) => Err(NodeError::custom("Source is not mutable")),
        }
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
            _ => Err(NodeError::custom(
                "Source is not a mutable BattleSimulation",
            )),
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
            _ => Err(NodeError::custom(
                "Source is not a mutable BattleSimulation",
            )),
        }
    }

    pub fn get_rng(&mut self) -> Option<&mut ChaCha8Rng> {
        match self {
            Self::BattleMut(battle, _) => Some(&mut battle.rng),
            _ => None,
        }
    }

    pub fn node_map(&self) -> Option<Arc<RwLock<SmartNodeMap>>> {
        match self {
            Self::Source(source) | Self::SourceMut(source) => {
                Some(source.read().unwrap().node_map())
            }
            _ => None,
        }
    }

    pub fn links(&self) -> Option<Arc<RwLock<NodeLinks>>> {
        match self {
            Self::Source(source) | Self::SourceMut(source) => Some(source.read().unwrap().links()),
            _ => None,
        }
    }
}

impl ContextSource for ClientSource<'_> {
    fn get_node_kind(&self, node_id: u64) -> NodeResult<NodeKind> {
        match self {
            Self::Source(source) | Self::SourceMut(source) => {
                let source_guard = source.read().unwrap();
                let node_map = source_guard.node_map();
                let node_map_guard = node_map.read().unwrap();
                node_map_guard
                    .get_node_kind(node_id)
                    .ok_or_else(|| NodeError::custom(format!("Node {} not found", node_id)))
            }
            Self::Db(db) => db.get_node_kind(node_id),
            _ => Err(NodeError::custom("Context does not support get_node_kind")),
        }
    }

    fn get_children(&self, from_id: u64) -> NodeResult<Vec<u64>> {
        match self {
            Self::Source(source) | Self::SourceMut(source) => {
                let source_guard = source.read().unwrap();
                let links = source_guard.links();
                let links_guard = links.read().unwrap();
                let mut children = links_guard.get_owned_children(from_id);
                children.extend(links_guard.get_ref_children(from_id));
                Ok(children)
            }
            Self::Db(db) => db.get_children(from_id),
            _ => Ok(Vec::new()),
        }
    }

    fn get_children_of_kind(&self, from_id: u64, kind: NodeKind) -> NodeResult<Vec<u64>> {
        match self {
            Self::Source(source) | Self::SourceMut(source) => {
                let source_guard = source.read().unwrap();
                let links = source_guard.links();
                let node_map = source_guard.node_map();
                let links_guard = links.read().unwrap();
                let node_map_guard = node_map.read().unwrap();

                let mut children = links_guard.get_owned_children(from_id);
                children.extend(links_guard.get_ref_children(from_id));

                Ok(children
                    .into_iter()
                    .filter(|&child_id| {
                        node_map_guard
                            .get_node_kind(child_id)
                            .map(|k| k == kind)
                            .unwrap_or(false)
                    })
                    .collect())
            }
            Self::Db(db) => db.get_children_of_kind(from_id, kind),
            _ => Ok(Vec::new()),
        }
    }

    fn get_parents(&self, to_id: u64) -> NodeResult<Vec<u64>> {
        match self {
            Self::Source(source) | Self::SourceMut(source) => {
                let source_guard = source.read().unwrap();
                let links = source_guard.links();
                let links_guard = links.read().unwrap();
                let mut parents = Vec::new();
                if let Some(owned_parent) = links_guard.get_owned_parent(to_id) {
                    parents.push(owned_parent);
                }
                parents.extend(links_guard.get_ref_parents(to_id));
                Ok(parents)
            }
            Self::Db(db) => db.get_parents(to_id),
            _ => Ok(Vec::new()),
        }
    }

    fn get_parents_of_kind(&self, to_id: u64, kind: NodeKind) -> NodeResult<Vec<u64>> {
        match self {
            Self::Source(source) | Self::SourceMut(source) => {
                let source_guard = source.read().unwrap();
                let links = source_guard.links();
                let node_map = source_guard.node_map();
                let links_guard = links.read().unwrap();
                let node_map_guard = node_map.read().unwrap();

                let mut parents = Vec::new();
                if let Some(owned_parent) = links_guard.get_owned_parent(to_id) {
                    parents.push(owned_parent);
                }
                parents.extend(links_guard.get_ref_parents(to_id));

                Ok(parents
                    .into_iter()
                    .filter(|&parent_id| {
                        node_map_guard
                            .get_node_kind(parent_id)
                            .map(|k| k == kind)
                            .unwrap_or(false)
                    })
                    .collect())
            }
            Self::Db(db) => db.get_parents_of_kind(to_id, kind),
            _ => Ok(Vec::new()),
        }
    }

    fn add_link(&mut self, from_id: u64, to_id: u64) -> NodeResult<()> {
        match self {
            Self::SourceMut(source) => {
                let mut source_guard = source.write().unwrap();
                let world_guard = source_guard.world();
                let mut world = world_guard.write().unwrap();
                let node_map_guard = source_guard.node_map();
                let node_map = node_map_guard.read().unwrap();
                let links_guard = source_guard.links();
                let mut links = links_guard.write().unwrap();

                // Determine link type and add appropriately
                if let Some(parent_kind) = node_map.get_node_kind(from_id) {
                    if let Some(child_kind) = node_map.get_node_kind(to_id) {
                        // TODO: Implement has_owned_child() method in NodeKind
                        if false {
                            // parent_kind.has_owned_child(&child_kind) {
                            links.add_owned_link(&mut world, from_id, to_id, &node_map);
                        // TODO: Implement has_ref_child() method in NodeKind
                        } else if false {
                            // parent_kind.has_ref_child(&child_kind) {
                            links.add_ref_link(&mut world, from_id, to_id, &node_map);
                        }
                        return Ok(());
                    }
                }
                Err(NodeError::custom("Could not determine link type"))
            }
            _ => Err(NodeError::custom("Context does not support add_link")),
        }
    }

    fn remove_link(&mut self, from_id: u64, to_id: u64) -> NodeResult<()> {
        match self {
            Self::SourceMut(source) => {
                let mut source_guard = source.write().unwrap();
                let world_guard = source_guard.world();
                let mut world = world_guard.write().unwrap();
                let node_map_guard = source_guard.node_map();
                let node_map = node_map_guard.read().unwrap();
                let links_guard = source_guard.links();
                let mut links = links_guard.write().unwrap();

                // Remove both types (will be no-op if not present)
                links.remove_owned_link(&mut world, from_id, to_id, &node_map);
                links.remove_ref_link(&mut world, from_id, to_id, &node_map);
                Ok(())
            }
            _ => Err(NodeError::custom("Context does not support remove_link")),
        }
    }

    fn is_linked(&self, from_id: u64, to_id: u64) -> NodeResult<bool> {
        match self {
            Self::Source(source) | Self::SourceMut(source) => {
                let source_guard = source.read().unwrap();
                let links = source_guard.links();
                let links_guard = links.read().unwrap();
                Ok(links_guard.has_owned_link(from_id, to_id)
                    || links_guard.has_ref_link(from_id, to_id))
            }
            Self::Db(db) => db.is_linked(from_id, to_id),
            _ => Ok(false),
        }
    }

    fn insert_node(&mut self, id: u64, owner: u64, kind: NodeKind, data: String) -> NodeResult<()> {
        match self {
            Self::SourceMut(source) => {
                let mut source_guard = source.write().unwrap();
                source_guard.insert_node(id, owner, kind, data);
                Ok(())
            }
            _ => Err(NodeError::custom("Context does not support insert_node")),
        }
    }

    fn delete_node(&mut self, id: u64) -> NodeResult<()> {
        match self {
            Self::SourceMut(source) => {
                let mut source_guard = source.write().unwrap();
                source_guard.delete_node(id);
                Ok(())
            }
            _ => Err(NodeError::custom("Context does not support delete_node")),
        }
    }

    fn get_var_direct(&self, node_id: u64, var: VarName) -> NodeResult<VarValue> {
        match self {
            Self::Db(db) => db.get_var_direct(node_id, var),
            _ => Err(NodeError::custom("Context does not support get_var_direct")),
        }
    }

    fn set_var(&mut self, node_id: u64, _var: VarName, _value: VarValue) -> NodeResult<()> {
        match self {
            Self::BattleMut(..) => {
                // Battle context could support this with node state history
                Ok(())
            }
            _ => Err(NodeError::custom("Context does not support set_var")),
        }
    }
}

impl ContextSource for &mut ClientSource<'_> {
    fn get_node_kind(&self, node_id: u64) -> NodeResult<NodeKind> {
        (**self).get_node_kind(node_id)
    }

    fn get_children(&self, from_id: u64) -> NodeResult<Vec<u64>> {
        (**self).get_children(from_id)
    }

    fn get_children_of_kind(&self, from_id: u64, kind: NodeKind) -> NodeResult<Vec<u64>> {
        (**self).get_children_of_kind(from_id, kind)
    }

    fn get_parents(&self, to_id: u64) -> NodeResult<Vec<u64>> {
        (**self).get_parents(to_id)
    }

    fn get_parents_of_kind(&self, to_id: u64, kind: NodeKind) -> NodeResult<Vec<u64>> {
        (**self).get_parents_of_kind(to_id, kind)
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

    fn get_var_direct(&self, node_id: u64, var: VarName) -> NodeResult<VarValue> {
        (**self).get_var_direct(node_id, var)
    }

    fn set_var(&mut self, node_id: u64, var: VarName, value: VarValue) -> NodeResult<()> {
        (**self).set_var(node_id, var, value)
    }
}
