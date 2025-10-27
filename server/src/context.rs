use serde::de::DeserializeOwned;
use spacetimedb::StdbRng;

use super::*;

/// Trait for server-side node data sources
pub trait ServerSourceTrait {
    fn insert_node<T: Node>(&mut self, node: T) -> NodeResult<()>;
}

/// Server-side source implementation
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

    pub fn rng(&self) -> &StdbRng {
        self.ctx.rng()
    }

    pub fn load<T>(&self, node_id: u64) -> NodeResult<T>
    where
        T: Node + DeserializeOwned,
    {
        let tnode = self
            .ctx
            .db
            .nodes_world()
            .id()
            .find(&node_id)
            .ok_or_else(|| NodeError::not_found(node_id))?;

        let expected_kind = T::kind_s();
        if tnode.kind.to_kind() != expected_kind {
            return Err(NodeError::invalid_kind(expected_kind, tnode.kind.to_kind()));
        }

        tnode.to_node::<T>()
    }
}

impl<'a> ServerSourceTrait for ServerSource<'a> {
    fn insert_node<T: Node>(&mut self, node: T) -> NodeResult<()> {
        let id = node.id();
        let owner = node.owner();
        let kind = T::kind_s();
        let data = node.get_data();

        self.ctx
            .db
            .nodes_world()
            .insert(TNode::new(id, owner, kind, data));

        Ok(())
    }
}

impl<'a> ContextSource for ServerSource<'a> {
    fn get_var(&self, node_id: u64, var: VarName) -> NodeResult<VarValue> {
        let kind = self.get_node_kind(node_id)?;
        node_kind_match!(kind, {
            let tnode = self
                .ctx
                .db
                .nodes_world()
                .id()
                .find(&node_id)
                .ok_or_else(|| NodeError::not_found(node_id))?;
            let node: NodeType = tnode.to_node()?;
            node.get_var(var)
        })
    }

    fn set_var(&mut self, node_id: u64, var: VarName, value: VarValue) -> NodeResult<()> {
        let kind = self.get_node_kind(node_id)?;
        node_kind_match!(kind, {
            let tnode = self
                .ctx
                .db
                .nodes_world()
                .id()
                .find(&node_id)
                .ok_or_else(|| NodeError::not_found(node_id))?;
            let mut node: NodeType = tnode.to_node()?;
            node.set_var(var, value)?;
            let data = node.get_data();
            self.ctx
                .db
                .nodes_world()
                .id()
                .update(TNode::new(node_id, node.owner(), kind, data));
        });
        Ok(())
    }

    fn var_updated(&mut self, _node_id: u64, _var: VarName, _value: VarValue) {
        // Server doesn't need history tracking like client battle simulations
    }

    fn get_node_kind(&self, node_id: u64) -> NodeResult<NodeKind> {
        node_id.kind(&self.ctx).ok_or(NodeError::not_found(node_id))
    }

    fn get_children(&self, node_id: u64) -> NodeResult<Vec<u64>> {
        Ok(node_id
            .collect_children_recursive(&self.ctx)
            .into_iter()
            .collect())
    }

    fn get_children_of_kind(&self, node_id: u64, kind: NodeKind) -> NodeResult<Vec<u64>> {
        Ok(node_id
            .collect_kind_children(&self.ctx, kind)
            .into_iter()
            .collect())
    }

    fn get_parents(&self, node_id: u64) -> NodeResult<Vec<u64>> {
        Ok(node_id
            .collect_parents_recursive(&self.ctx)
            .into_iter()
            .collect())
    }

    fn get_parents_of_kind(&self, node_id: u64, kind: NodeKind) -> NodeResult<Vec<u64>> {
        Ok(node_id.collect_kind_parents(&self.ctx, kind))
    }

    fn add_link(&mut self, parent_id: u64, child_id: u64) -> NodeResult<()> {
        parent_id.add_child(&self.ctx, child_id)
    }

    fn remove_link(&mut self, parent_id: u64, child_id: u64) -> NodeResult<()> {
        parent_id.remove_child(&self.ctx, child_id)
    }

    fn clear_links(&mut self, node_id: u64) -> NodeResult<()> {
        // Remove all child links
        let children = self.get_children(node_id)?;
        for child_id in children {
            self.remove_link(node_id, child_id)?;
        }

        // Remove all parent links
        let parents = self.get_parents(node_id)?;
        for parent_id in parents {
            self.remove_link(parent_id, node_id)?;
        }

        Ok(())
    }

    fn is_linked(&self, parent_id: u64, child_id: u64) -> NodeResult<bool> {
        Ok(parent_id.has_child(&self.ctx, child_id))
    }

    fn delete_node(&mut self, node_id: u64) -> NodeResult<()> {
        self.clear_links(node_id)?;
        self.ctx.db.nodes_world().id().delete(node_id);
        Ok(())
    }
}

// Convenience type alias
pub type ServerContext<'a> = Context<ServerSource<'a>>;

// Extension trait for ReducerContext
pub trait ReducerContextExt {
    fn with_context<F>(&self, f: F) -> Result<(), String>
    where
        F: FnOnce(&mut ServerContext) -> NodeResult<()>;

    fn as_context(&self) -> ServerContext;
}

impl ReducerContextExt for ReducerContext {
    #[track_caller]
    fn with_context<F>(&self, f: F) -> Result<(), String>
    where
        F: FnOnce(&mut ServerContext) -> NodeResult<()>,
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

// Extension trait for ServerContext
pub trait ServerContextExt {
    fn load<T>(&self, node_id: u64) -> NodeResult<T>
    where
        T: Node + DeserializeOwned;

    fn load_many<T>(&self, ids: &[u64]) -> NodeResult<Vec<T>>
    where
        T: Node + DeserializeOwned;

    fn load_linked<T>(&self, from_id: u64) -> NodeResult<Vec<T>>
    where
        T: Node + DeserializeOwned;

    fn load_top_child<T>(&self, from_id: u64) -> NodeResult<Option<T>>
    where
        T: Node + DeserializeOwned;

    fn load_top_parent<T>(&self, id: u64) -> NodeResult<Option<T>>
    where
        T: Node + DeserializeOwned;

    fn rctx(&self) -> &ReducerContext;
    fn rng(&self) -> &StdbRng;
}

impl<'a> ServerContextExt for ServerContext<'a> {
    fn load<T>(&self, node_id: u64) -> NodeResult<T>
    where
        T: Node + DeserializeOwned,
    {
        self.source().load(node_id)
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
        if let Some(id) = from_id.top_child(self.source().ctx, kind) {
            Ok(Some(self.load::<T>(id)?))
        } else {
            Ok(None)
        }
    }

    fn load_top_parent<T>(&self, id: u64) -> NodeResult<Option<T>>
    where
        T: Node + DeserializeOwned,
    {
        let kind = T::kind_s();
        if let Some(id) = id.top_parent(self.source().ctx, kind) {
            Ok(Some(self.load::<T>(id)?))
        } else {
            Ok(None)
        }
    }

    fn rctx(&self) -> &ReducerContext {
        self.source().rctx()
    }

    fn rng(&self) -> &StdbRng {
        self.source().rng()
    }
}

// Helper macro for converting NodeError to String
#[macro_export]
macro_rules! node_err_to_string {
    ($result:expr) => {
        $result.map_err(|e: schema::NodeError| e.to_string())
    };
}

// Extension trait for Result<T, NodeError>
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
