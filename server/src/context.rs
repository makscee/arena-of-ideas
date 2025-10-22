use serde::de::DeserializeOwned;
use spacetimedb::StdbRng;

use super::*;

/// ContextSource implementation for SpacetimeDB
pub struct ServerSource<'a> {
    ctx: &'a ReducerContext,
}

impl<'a> ServerSource<'a> {
    pub fn new(ctx: &'a ReducerContext) -> Self {
        Self { ctx }
    }

    pub fn rctx(&self) -> &ReducerContext {
        self.ctx
    }
}

impl<'a> ContextSource for ServerSource<'a> {
    fn get_node_kind(&self, id: u64) -> NodeResult<NodeKind> {
        id.kind(&self.ctx).ok_or(NodeError::not_found(id))
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
        from_id.add_child(&self.ctx, to_id)
    }

    fn remove_link(&mut self, from_id: u64, to_id: u64) -> NodeResult<()> {
        from_id.remove_child(&self.ctx, to_id)
    }

    fn is_linked(&self, from_id: u64, to_id: u64) -> NodeResult<bool> {
        Ok(from_id.has_child(&self.ctx, to_id))
    }

    fn insert_node(&mut self, id: u64, owner: u64, kind: NodeKind, data: String) -> NodeResult<()> {
        match self
            .rctx()
            .db
            .nodes_world()
            .try_insert(TNode::new(id, owner, kind, data.clone()))
        {
            Ok(_) => {}
            Err(_) => {
                self.rctx()
                    .db
                    .nodes_world()
                    .id()
                    .update(TNode::new(id, owner, kind, data));
            }
        }
        Ok(())
    }

    fn delete_node(&mut self, id: u64) -> NodeResult<()> {
        self.rctx().db.nodes_world().id().delete(id);
        Ok(())
    }

    fn get_var_direct(&self, node_id: u64, var: VarName) -> NodeResult<VarValue> {
        let kind = self.get_node_kind(node_id)?;
        node_kind_match!(kind, {
            let node: NodeType = {
                let tnode = self
                    .ctx
                    .db
                    .nodes_world()
                    .id()
                    .find(&node_id)
                    .to_not_found()?;
                tnode.to_node()?
            };
            node.get_var(var)
        })
    }

    fn set_var(&mut self, _: u64, _: VarName, _: VarValue) -> NodeResult<()> {
        Ok(())
    }
}

impl<'a> ContextSource for &mut ServerSource<'a> {
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

/// Extension trait for Context to load nodes in server
pub trait ServerContextExt<S: ContextSource> {
    /// Load a node by ID with type checking
    fn load<T>(&self, id: u64) -> NodeResult<T>
    where
        T: Node + DeserializeOwned;

    /// Load multiple nodes
    fn load_many<T>(&self, ids: &[u64]) -> NodeResult<Vec<T>>
    where
        T: Node + DeserializeOwned;

    /// Load linked nodes
    fn load_linked<T>(&self, from_id: u64) -> NodeResult<Vec<T>>
    where
        T: Node + DeserializeOwned;

    /// Load top child node
    fn load_top_child<T>(&self, from_id: u64) -> NodeResult<Option<T>>
    where
        T: Node + DeserializeOwned;

    /// Load parent nodes
    fn load_parents<T>(&self, id: u64) -> NodeResult<Vec<T>>
    where
        T: Node + DeserializeOwned;

    /// Load top parent node
    fn load_top_parent<T>(&self, id: u64) -> NodeResult<Option<T>>
    where
        T: Node + DeserializeOwned;
    fn rctx(&self) -> &ReducerContext;
    fn rng(&self) -> &StdbRng;
}

impl<'a> ServerContextExt<ServerSource<'a>> for Context<ServerSource<'a>> {
    fn load<T>(&self, id: u64) -> NodeResult<T>
    where
        T: Node + DeserializeOwned,
    {
        let node = id
            .load_tnode_err(&self.source().ctx)
            .map_err(|e| NodeError::load_error(format!("Failed to load TNode: {}", e)))?;

        let expected_kind = T::kind_s();
        if node.kind.to_kind() != expected_kind {
            return Err(NodeError::invalid_kind(expected_kind, node.kind.to_kind()));
        }
        node.to_node::<T>()
    }

    fn load_many<T>(&self, ids: &[u64]) -> NodeResult<Vec<T>>
    where
        T: Node + DeserializeOwned,
    {
        ids.iter().map(|id| self.load::<T>(*id)).collect()
    }

    fn load_linked<T>(&self, from_id: u64) -> NodeResult<Vec<T>>
    where
        T: Node + DeserializeOwned,
    {
        let kind = T::kind_s();
        let ids = self.get_children_of_kind(from_id, kind)?;
        self.load_many(&ids)
    }

    fn load_top_child<T>(&self, from_id: u64) -> NodeResult<Option<T>>
    where
        T: Node + DeserializeOwned,
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
        T: Node + DeserializeOwned,
    {
        let kind = T::kind_s();
        let ids = self.get_parents_of_kind(id, kind)?;
        self.load_many(&ids)
    }

    fn load_top_parent<T>(&self, id: u64) -> NodeResult<Option<T>>
    where
        T: Node + DeserializeOwned,
    {
        let kind = T::kind_s();
        if let Some(id) = id.top_parent(&self.source().ctx, kind) {
            Ok(Some(self.load::<T>(id)?))
        } else {
            Ok(None)
        }
    }

    fn rctx(&self) -> &ReducerContext {
        self.source().rctx()
    }

    fn rng(&self) -> &StdbRng {
        self.rctx().rng()
    }
}

/// Extension for using Context with ReducerContext
pub trait ReducerContextExt {
    /// Execute with a context
    fn with_context<F>(&self, f: F) -> Result<(), String>
    where
        F: FnOnce(&mut Context<ServerSource>) -> Result<(), NodeError>;
    fn as_context(&self) -> ServerContext;
}

impl ReducerContextExt for ReducerContext {
    #[track_caller]
    fn with_context<F>(&self, f: F) -> Result<(), String>
    where
        F: FnOnce(&mut Context<ServerSource>) -> Result<(), NodeError>,
    {
        let source = ServerSource::new(self);
        let location = std::panic::Location::caller();
        Context::exec(source, f)
            .map_err(|e| format!("{} (at {}:{})", e, location.file(), location.line()))
    }

    fn as_context(&self) -> ServerContext {
        Context::new(ServerSource::new(self))
    }
}

/// Macro for converting NodeError to String (location already included in NodeError)
#[macro_export]
macro_rules! node_err_to_string {
    ($result:expr) => {
        $result.map_err(|e: schema::NodeError| e.to_string())
    };
}

/// Extension trait for Result<T, NodeError> to easily convert to Result<T, String> with location
pub trait ServerNodeResultExt<T> {
    fn to_server_result(self) -> Result<T, String>;
}

impl<T> ServerNodeResultExt<T> for NodeResult<T> {
    #[track_caller]
    fn to_server_result(self) -> Result<T, String> {
        self.map_err(|e| {
            let location = std::panic::Location::caller();
            format!("{} (at {}:{})", e, location.file(), location.line())
        })
    }
}

pub type ServerContext<'a> = Context<ServerSource<'a>>;
