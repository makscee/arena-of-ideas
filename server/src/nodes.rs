use super::*;

use schema::*;

include!(concat!(env!("OUT_DIR"), "/server_nodes.rs"));

#[allow(unused)]
pub trait ServerNode: Sized + schema::Node {
    fn load(ctx: &ServerContext, id: u64) -> NodeResult<Self> {
        let kind = Self::kind_s().to_string();
        let node: TNode = ctx
            .source()
            .reducer_context()
            .db
            .nodes_world()
            .id()
            .find(id)?;
        if node.kind == kind {
            node.to_node()
        } else {
            Err(NodeError::InvalidKind {
                expected: kind.to_kind(),
                actual: node.kind(),
            })
        }
    }
    fn save(&self, ctx: &ReducerContext);
    fn clone_self(&self, ctx: &ReducerContext, owner: u64) -> Self;
    fn clone(&self, ctx: &ReducerContext, owner: u64) -> Self;
    fn insert(mut self, ctx: &ReducerContext) -> Self {
        if self.id() == 0 {
            self.set_id(next_id(ctx));
        }
        let node = self.to_tnode();
        debug!("insert {node:?}");
        match ctx.db.nodes_world().try_insert(node.clone()) {
            Ok(_) => {}
            Err(e) => error!("Insert of {node:?} failed: {e}"),
        }
        self
    }
    fn update(&self, ctx: &ReducerContext) {
        if self.id() == 0 {
            panic!("Node id not set");
        }
        let node = self.to_tnode();
        ctx.db.nodes_world().id().update(node);
    }
    fn delete(&self, ctx: &ReducerContext) {
        if self.id() == 0 {
            panic!("Node id not set");
        }
        ctx.db.node_links().child().delete(self.id());
        ctx.db.node_links().parent().delete(self.id());
        TNode::delete_by_id(ctx, self.id());
    }
    fn to_tnode(&self) -> TNode {
        TNode::new(self.id(), self.owner(), self.kind(), self.get_data())
    }
}
