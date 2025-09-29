pub use schema_v2::*;

// Re-export from schema
pub use schema_v2::{Context, ContextLayer, ContextSource};

// Re-export generated code
include!(concat!(env!("OUT_DIR"), "/server_nodes.rs"));

use serde_json;
use spacetimedb::{Identity, ReducerContext};

use server::nodes_table::*;

/// ContextSource implementation for SpacetimeDB
pub struct ServerSource<'a> {
    ctx: &'a ReducerContext,
}

impl<'a> ServerSource<'a> {
    pub fn new(ctx: &'a ReducerContext) -> Self {
        Self { ctx }
    }

    pub fn reducer_context(&self) -> &ReducerContext {
        self.ctx
    }
}

impl<'a> ContextSource for ServerSource<'a> {
    fn get_node_kind(&self, id: u64) -> NodeResult<NodeKind> {
        id.kind().ok_or(NodeError::NotFound(id))
    }

    fn get_children(&self, from_id: u64) -> NodeResult<Vec<u64>> {
        Ok(from_id.collect_children_recursive())
    }

    fn get_children_of_kind(&self, from_id: u64, kind: NodeKind) -> NodeResult<Vec<u64>> {
        Ok(from_id.collect_kind_children(kind))
    }

    fn get_parents(&self, id: u64) -> NodeResult<Vec<u64>> {
        Ok(id.collect_parents_recursive())
    }

    fn get_parents_of_kind(&self, id: u64, kind: NodeKind) -> NodeResult<Vec<u64>> {
        Ok(id.collect_kind_parents(kind))
    }

    fn add_link(&mut self, from_id: u64, to_id: u64) -> NodeResult<()> {
        from_id
            .add_child(to_id)
            .map_err(|e| NodeError::ContextError(anyhow::anyhow!("Failed to add link: {}", e)))
    }

    fn remove_link(&mut self, from_id: u64, to_id: u64) -> NodeResult<()> {
        from_id
            .remove_child(to_id)
            .map_err(|e| NodeError::ContextError(anyhow::anyhow!("Failed to remove link: {}", e)))
    }

    fn is_linked(&self, from_id: u64, to_id: u64) -> NodeResult<bool> {
        Ok(from_id.has_child(to_id))
    }
}

/// Extension trait for Context to load nodes in server
pub trait ServerContextExt<S: ContextSource> {
    /// Load a node by ID with type checking
    fn load<T>(&self, id: u64) -> NodeResult<T>
    where
        T: 'static + HasNodeKind + serde::de::DeserializeOwned;

    /// Load multiple nodes
    fn load_many<T>(&self, ids: &[u64]) -> NodeResult<Vec<T>>
    where
        T: 'static + HasNodeKind + serde::de::DeserializeOwned;

    /// Load linked nodes
    fn load_children<T>(&self, from_id: u64) -> NodeResult<Vec<T>>
    where
        T: 'static + HasNodeKind + serde::de::DeserializeOwned;

    /// Load top child node
    fn load_top_child<T>(&self, from_id: u64) -> NodeResult<Option<T>>
    where
        T: 'static + HasNodeKind + serde::de::DeserializeOwned;

    /// Load parent nodes
    fn load_parents<T>(&self, id: u64) -> NodeResult<Vec<T>>
    where
        T: 'static + HasNodeKind + serde::de::DeserializeOwned;

    /// Load top parent node
    fn load_top_parent<T>(&self, id: u64) -> NodeResult<Option<T>>
    where
        T: 'static + HasNodeKind + serde::de::DeserializeOwned;
}

impl<'a> ServerContextExt<ServerSource<'a>> for Context<ServerSource<'a>> {
    fn load<T>(&self, id: u64) -> NodeResult<T>
    where
        T: 'static + HasNodeKind + serde::de::DeserializeOwned,
    {
        let node = id
            .load_tnode_err()
            .map_err(|e| NodeError::LoadError(format!("Failed to load TNode: {}", e)))?;

        let expected_kind = T::node_kind();
        if node.kind != expected_kind {
            return Err(NodeError::InvalidKind {
                expected: expected_kind,
                actual: node.kind,
            });
        }

        serde_json::from_slice::<T>(&node.data)
            .map_err(|e| NodeError::LoadError(format!("Failed to deserialize: {}", e)))
    }

    fn load_many<T>(&self, ids: &[u64]) -> NodeResult<Vec<T>>
    where
        T: 'static + HasNodeKind + serde::de::DeserializeOwned,
    {
        ids.iter().map(|id| self.load::<T>(*id)).collect()
    }

    fn load_children<T>(&self, from_id: u64) -> NodeResult<Vec<T>>
    where
        T: 'static + HasNodeKind + serde::de::DeserializeOwned,
    {
        let kind = T::node_kind();
        let ids = self.get_children_of_kind(from_id, kind)?;
        self.load_many(&ids)
    }

    fn load_top_child<T>(&self, from_id: u64) -> NodeResult<Option<T>>
    where
        T: 'static + HasNodeKind + serde::de::DeserializeOwned,
    {
        let kind = T::node_kind();
        if let Some(id) = from_id.top_child(kind) {
            Ok(Some(self.load::<T>(id)?))
        } else {
            Ok(None)
        }
    }

    fn load_parents<T>(&self, id: u64) -> NodeResult<Vec<T>>
    where
        T: 'static + HasNodeKind + serde::de::DeserializeOwned,
    {
        let kind = T::node_kind();
        let ids = self.get_parents_of_kind(id, kind)?;
        self.load_many(&ids)
    }

    fn load_top_parent<T>(&self, id: u64) -> NodeResult<Option<T>>
    where
        T: 'static + HasNodeKind + serde::de::DeserializeOwned,
    {
        let kind = T::node_kind();
        if let Some(id) = id.top_parent(kind) {
            Ok(Some(self.load::<T>(id)?))
        } else {
            Ok(None)
        }
    }
}

/// Extension for using Context with ReducerContext
pub trait ReducerContextExt {
    /// Execute with a context
    fn with_context<R, F>(&self, f: F) -> R
    where
        F: FnOnce(&mut Context<ServerSource>) -> R;
}

impl ReducerContextExt for ReducerContext {
    fn with_context<R, F>(&self, f: F) -> R
    where
        F: FnOnce(&mut Context<ServerSource>) -> R,
    {
        let source = ServerSource::new(self);
        Context::exec(source, f)
    }
}

/// Initialize the SpacetimeDB module
// TODO: Add proper SpacetimeDB init when implementing actual tables
pub fn init() {
    // Initialize server state if needed
}

/// Helper module for node operations
pub mod node_ops {
    use super::*;

    /// Create a new node
    pub fn create_node(
        _ctx: &ReducerContext,
        owner: Identity,
        kind: NodeKind,
        data: Vec<u8>,
    ) -> NodeResult<u64> {
        let node = TNode::new(0, owner, kind, data);
        let inserted = node.insert().map_err(|e| {
            NodeError::ContextError(anyhow::anyhow!("Failed to insert node: {}", e))
        })?;
        Ok(inserted.id)
    }

    /// Update node data
    pub fn update_node(ctx: &ReducerContext, id: u64, data: Vec<u8>) -> NodeResult<()> {
        if let Some(mut node) = id.load_tnode(ctx) {
            node.data = data;
            node.update(ctx).map_err(|e| {
                NodeError::ContextError(anyhow::anyhow!("Failed to update node: {}", e))
            })?;
        }
        Ok(())
    }

    /// Delete a node and all its children recursively
    pub fn delete_node_recursive(_ctx: &ReducerContext, id: u64) -> NodeResult<()> {
        TNode::delete_by_id_recursive(&id)
            .map_err(|e| NodeError::ContextError(anyhow::anyhow!("Failed to delete node: {}", e)))
    }

    /// Delete a node without deleting children
    pub fn delete_node_only(_ctx: &ReducerContext, id: u64) -> NodeResult<()> {
        TNode::delete_by_id(_ctx, &id)
            .map_err(|e| NodeError::ContextError(anyhow::anyhow!("Failed to delete node: {}", e)))
    }

    /// Solidify a link between two nodes
    pub fn solidify_link(ctx: &ReducerContext, parent_id: u64, child_id: u64) -> NodeResult<()> {
        TNodeLink::solidify(ctx, parent_id, child_id)
            .map_err(|e| NodeError::ContextError(anyhow::anyhow!("Failed to solidify link: {}", e)))
    }

    /// Get all nodes of a specific kind
    pub fn get_nodes_by_kind(_ctx: &ReducerContext, kind: NodeKind) -> Vec<TNode> {
        TNode::filter_by_kind(&kind).collect()
    }

    /// Get all nodes owned by a specific identity
    pub fn get_nodes_by_owner(_ctx: &ReducerContext, owner: Identity) -> Vec<TNode> {
        TNode::filter_by_owner(&owner).collect()
    }

    /// Get all nodes of a specific kind owned by a specific identity
    pub fn get_nodes_by_kind_and_owner(
        _ctx: &ReducerContext,
        kind: NodeKind,
        owner: Identity,
    ) -> Vec<TNode> {
        TNode::collect_kind_owner(kind, owner)
    }
}
