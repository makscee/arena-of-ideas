use super::*;
use crate::stdb::RemoteTables;

pub struct DbSource<'a> {
    db: &'a RemoteTables,
}

impl<'a> DbSource<'a> {
    pub fn new(db: &'a RemoteTables) -> Self {
        Self { db }
    }
}

impl WorldContextExt for RemoteTables {
    fn with_context<R, F>(&self, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut Context<ClientSource<'_>>) -> NodeResult<R>,
    {
        let source = ClientSource::new_db(self);
        Context::exec(source, f)
    }

    fn as_context(&self) -> Context<ClientSource<'_>> {
        Context::new(ClientSource::Db(self))
    }

    fn with_context_mut<R, F>(&mut self, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut Context<ClientSource<'_>>) -> NodeResult<R>,
    {
        let source = ClientSource::new_db(self);
        Context::exec(source, f)
    }

    fn as_context_mut(&mut self) -> Context<ClientSource<'_>> {
        Context::new(ClientSource::Db(self))
    }
}

impl<'a> ContextSource for DbSource<'a> {
    fn get_node_kind(&self, node_id: u64) -> NodeResult<NodeKind> {
        self.db
            .nodes_world()
            .id()
            .find(&node_id)
            .and_then(|n| NodeKind::try_from(n.kind.as_str()).ok())
            .ok_or_else(|| NodeError::custom("Node not found"))
    }

    fn get_children(&self, from_id: u64) -> NodeResult<Vec<u64>> {
        Ok(self
            .db
            .node_links()
            .iter()
            .filter(|l| l.parent == from_id)
            .map(|l| l.child)
            .collect())
    }

    fn get_children_of_kind(&self, from_id: u64, kind: NodeKind) -> NodeResult<Vec<u64>> {
        let kind_str = kind.as_ref();
        Ok(self
            .db
            .node_links()
            .iter()
            .filter(|l| l.parent == from_id && l.child_kind == kind_str)
            .map(|l| l.child)
            .collect())
    }

    fn get_parents(&self, to_id: u64) -> NodeResult<Vec<u64>> {
        Ok(self
            .db
            .node_links()
            .iter()
            .filter(|l| l.child == to_id)
            .map(|l| l.parent)
            .collect())
    }

    fn get_parents_of_kind(&self, to_id: u64, kind: NodeKind) -> NodeResult<Vec<u64>> {
        let kind_str = kind.as_ref();
        Ok(self
            .db
            .node_links()
            .iter()
            .filter(|l| l.child == to_id && l.parent_kind == kind_str)
            .map(|l| l.parent)
            .collect())
    }

    fn add_link(&mut self, _from_id: u64, _to_id: u64) -> NodeResult<()> {
        Err(NodeError::custom("Cannot modify DB source"))
    }

    fn remove_link(&mut self, _from_id: u64, _to_id: u64) -> NodeResult<()> {
        Err(NodeError::custom("Cannot modify DB source"))
    }

    fn is_linked(&self, from_id: u64, to_id: u64) -> NodeResult<bool> {
        Ok(self.db.node_links().iter().any(|l| {
            (l.parent == from_id && l.child == to_id) || (l.child == from_id && l.parent == to_id)
        }))
    }

    fn insert_node(
        &mut self,
        _id: u64,
        _owner: u64,
        _kind: NodeKind,
        _data: String,
    ) -> NodeResult<()> {
        Err(NodeError::custom("Cannot modify DB source"))
    }

    fn delete_node(&mut self, _id: u64) -> NodeResult<()> {
        Err(NodeError::custom("Cannot modify DB source"))
    }

    fn get_var_direct(&self, id: u64, var: VarName) -> NodeResult<VarValue> {
        let tnode = self
            .db
            .nodes_world()
            .id()
            .find(&id)
            .ok_or_else(|| NodeError::custom("Node not found"))?;

        let kind = NodeKind::try_from(tnode.kind.as_str())
            .map_err(|_| NodeError::custom("Invalid node kind"))?;

        node_kind_match!(kind, {
            let node = tnode.to_node::<NodeType>()?;
            node.get_var(var)
        })
    }

    fn set_var(&mut self, _id: u64, _var: VarName, _value: VarValue) -> NodeResult<()> {
        Err(NodeError::custom("Cannot modify DB source"))
    }
}
