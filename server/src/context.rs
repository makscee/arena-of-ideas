use crate::nodes_table::*;
use itertools::Itertools;
use schema::{Context, ContextLayer, ContextSource, NodeError, NodeKind, NodeResult, ToNodeKind};
use serde::de::DeserializeOwned;
use spacetimedb::{Identity, ReducerContext};

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
        id.kind(&self.ctx).ok_or(NodeError::NotFound(id))
    }

    fn get_children(&self, from_id: u64) -> NodeResult<Vec<u64>> {
        Ok(from_id
            .collect_children_recursive(&self.ctx)
            .into_iter()
            .collect_vec())
    }

    fn get_children_of_kind(&self, from_id: u64, kind: NodeKind) -> NodeResult<Vec<u64>> {
        Ok(from_id
            .collect_kind_children(&self.ctx, kind)
            .into_iter()
            .collect_vec())
    }

    fn get_parents(&self, id: u64) -> NodeResult<Vec<u64>> {
        Ok(id
            .collect_parents_recursive(&self.ctx)
            .into_iter()
            .collect_vec())
    }

    fn get_parents_of_kind(&self, id: u64, kind: NodeKind) -> NodeResult<Vec<u64>> {
        Ok(id.collect_kind_parents(&self.ctx, kind))
    }

    fn add_link(&mut self, from_id: u64, to_id: u64) -> NodeResult<()> {
        from_id
            .add_child(&self.ctx, to_id)
            .map_err(|e| NodeError::ContextError(anyhow::anyhow!("Failed to add link: {}", e)))
    }

    fn remove_link(&mut self, from_id: u64, to_id: u64) -> NodeResult<()> {
        from_id
            .remove_child(&self.ctx, to_id)
            .map_err(|e| NodeError::ContextError(anyhow::anyhow!("Failed to remove link: {}", e)))
    }

    fn is_linked(&self, from_id: u64, to_id: u64) -> NodeResult<bool> {
        Ok(from_id.has_child(&self.ctx, to_id))
    }
}

/// Extension trait for Context to load nodes in server
pub trait ServerContextExt<S: ContextSource> {
    /// Load a node by ID with type checking
    fn load<T>(&self, id: u64) -> NodeResult<T>
    where
        T: 'static + schema::Node + DeserializeOwned;

    /// Load multiple nodes
    fn load_many<T>(&self, ids: &[u64]) -> NodeResult<Vec<T>>
    where
        T: 'static + schema::Node + DeserializeOwned;

    /// Load linked nodes
    fn load_linked<T>(&self, from_id: u64) -> NodeResult<Vec<T>>
    where
        T: 'static + schema::Node + DeserializeOwned;

    /// Load top child node
    fn load_top_child<T>(&self, from_id: u64) -> NodeResult<Option<T>>
    where
        T: 'static + schema::Node + DeserializeOwned;

    /// Load parent nodes
    fn load_parents<T>(&self, id: u64) -> NodeResult<Vec<T>>
    where
        T: 'static + schema::Node + DeserializeOwned;

    /// Load top parent node
    fn load_top_parent<T>(&self, id: u64) -> NodeResult<Option<T>>
    where
        T: 'static + schema::Node + DeserializeOwned;
}

impl<'a> ServerContextExt<ServerSource<'a>> for Context<ServerSource<'a>> {
    fn load<T>(&self, id: u64) -> NodeResult<T>
    where
        T: 'static + schema::Node + DeserializeOwned,
    {
        let node = id
            .load_tnode_err(&self.source().ctx)
            .map_err(|e| NodeError::LoadError(format!("Failed to load TNode: {}", e)))?;

        let expected_kind = T::kind_s();
        if node.kind.to_kind() != expected_kind {
            return Err(NodeError::InvalidKind {
                expected: expected_kind,
                actual: node.kind.to_kind(),
            });
        }
        ron::from_str::<T>(&node.data)
            .map_err(|e| NodeError::LoadError(format!("Failed to deserialize: {}", e)))
    }

    fn load_many<T>(&self, ids: &[u64]) -> NodeResult<Vec<T>>
    where
        T: 'static + schema::Node + DeserializeOwned,
    {
        ids.iter().map(|id| self.load::<T>(*id)).collect()
    }

    fn load_linked<T>(&self, from_id: u64) -> NodeResult<Vec<T>>
    where
        T: 'static + schema::Node + DeserializeOwned,
    {
        let kind = T::kind_s();
        let ids = self.get_children_of_kind(from_id, kind)?;
        self.load_many(&ids)
    }

    fn load_top_child<T>(&self, from_id: u64) -> NodeResult<Option<T>>
    where
        T: 'static + schema::Node + DeserializeOwned,
    {
        let kind = T::kind_s();
        if let Some(id) = from_id.top_child(&self.source().ctx, kind) {
            Ok(Some(self.load::<T>(id)?))
        } else {
            Ok(None)
        }
    }

    fn load_parents<T>(&self, id: u64) -> NodeResult<Vec<T>>
    where
        T: 'static + schema::Node + DeserializeOwned,
    {
        let kind = T::kind_s();
        let ids = self.get_parents_of_kind(id, kind)?;
        self.load_many(&ids)
    }

    fn load_top_parent<T>(&self, id: u64) -> NodeResult<Option<T>>
    where
        T: 'static + schema::Node + DeserializeOwned,
    {
        let kind = T::kind_s();
        if let Some(id) = id.top_parent(&self.source().ctx, kind) {
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

pub type ServerContext<'a> = Context<ServerSource<'a>>;
