use super::*;

use schema::*;

include!(concat!(env!("OUT_DIR"), "/server_nodes.rs"));

#[allow(unused)]
pub trait ServerNode: Sized + schema::Node {
    fn save(self, ctx: &mut ServerContext) -> NodeResult<()>;
    fn load_components(&mut self, ctx: &ServerContext) -> NodeResult<&mut Self>;
    fn load_all(&mut self, ctx: &ServerContext) -> NodeResult<&mut Self>;
    fn load(source: &ServerSource, id: u64) -> NodeResult<Self> {
        let kind = Self::kind_s().to_string();
        let node: TNode = source
            .rctx()
            .db
            .nodes_world()
            .id()
            .find(id)
            .to_not_found_msg(format!("node#{id}"))?;
        if node.kind == kind {
            node.to_node()
        } else {
            Err(NodeError::invalid_kind(
                kind.to_kind(),
                NodeKind::from_str(&node.kind).unwrap_or(NodeKind::None),
            ))
        }
    }
    fn load_parent<T: ServerNode>(&self, ctx: &ServerContext) -> NodeResult<T> {
        let kind = T::kind_s();
        let parent_id = self
            .id()
            .get_kind_parent(ctx.rctx(), kind)
            .to_custom_err_fn(|| format!("{kind} parent of {} not found", self.id()))?;
        T::load(ctx.source(), parent_id)
    }
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
        for id in self.collect_owned_ids() {
            debug!("delete node {id} {}", self.kind());
            TNode::delete_by_id(ctx.rctx(), id);
        }
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
    fn remap_ids(mut self, ctx: &ServerContext) -> Self {
        let mut next_id = ctx.next_id();
        let mut id_map = std::collections::HashMap::new();
        self.reassign_ids(&mut next_id, &mut id_map);
        GlobalData::set_next_id(ctx.rctx(), next_id);
        self
    }
}
