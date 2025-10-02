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
            .find(id)
            .to_not_found_msg(format!("node#{id}"))?;
        if node.kind == kind {
            node.to_node()
        } else {
            Err(NodeError::InvalidKind {
                expected: kind.to_kind(),
                actual: node.kind(),
            })
        }
    }
    fn save(&self, ctx: &ServerContext);
    fn clone_self(&self, ctx: &ServerContext, owner: u64) -> Self;
    fn clone(&self, ctx: &ServerContext, owner: u64) -> Self;
    fn insert(mut self, ctx: &ServerContext) -> Self {
        if self.id() == 0 {
            self.set_id(next_id(ctx.rctx()));
        }
        let node = self.to_tnode();
        debug!("insert {node:?}");
        match ctx.rctx().db.nodes_world().try_insert(node.clone()) {
            Ok(_) => {}
            Err(e) => error!("Insert of {node:?} failed: {e}"),
        }
        self
    }
    fn update(&self, ctx: &ServerContext) {
        if self.id() == 0 {
            panic!("Node id not set");
        }
        let node = self.to_tnode();
        ctx.rctx().db.nodes_world().id().update(node);
    }
    fn delete(&self, ctx: &ServerContext) {
        if self.id() == 0 {
            panic!("Node id not set");
        }
        let ctx = ctx.rctx();
        ctx.db.node_links().child().delete(self.id());
        ctx.db.node_links().parent().delete(self.id());
        TNode::delete_by_id(ctx, self.id());
    }
    fn to_tnode(&self) -> TNode {
        TNode::new(self.id(), self.owner(), self.kind(), self.get_data())
    }
    fn collect_owner(ctx: &ServerContext, owner: u64) -> Vec<Self> {
        let kind = Self::kind_s().to_string();
        ctx.rctx()
            .db
            .nodes_world()
            .owner()
            .filter(owner)
            .filter_map(|n| {
                if n.kind == kind {
                    n.to_node::<Self>().ok()
                } else {
                    None
                }
            })
            .collect_vec()
    }
    fn delete_recursive(&self, ctx: &ServerContext) {
        todo!()
    }
    fn take(&mut self) -> Self {
        std::mem::take(self)
    }
    fn find_by_data(ctx: &ServerContext, data: &String) -> Option<Self> {
        let kind = Self::kind_s().to_string();
        ctx.rctx()
            .db
            .nodes_world()
            .data()
            .filter(data)
            .filter_map(|n| {
                if n.kind == kind {
                    n.to_node::<Self>().ok()
                } else {
                    None
                }
            })
            .next()
    }
}
