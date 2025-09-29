use super::*;

use schema::*;
use serde::{Deserialize, Serialize};
use serde::{
    de::{self, Visitor},
    ser::SerializeTuple,
};

include!(concat!(env!("OUT_DIR"), "/server_nodes.rs"));

#[allow(unused)]
pub trait ServerNode: Default + Sized + StringData + schema::Node {
    fn pack_fill(&self, pn: &mut PackedNodes);
    fn pack(&self) -> PackedNodes;
    fn unpack_id(id: u64, pn: &PackedNodes) -> Option<Self>;
    fn with_parts(&mut self, ctx: &ReducerContext) -> &mut Self;
    fn save(&self, ctx: &ReducerContext);
    fn clone_self(&self, ctx: &ReducerContext, owner: u64) -> Self;
    fn clone(&self, ctx: &ReducerContext, owner: u64) -> Self;
    fn collect_ids(&self) -> Vec<u64>;
    fn solidify_links(&self, ctx: &ReducerContext) -> Result<(), String>;
    fn delete_with_parts(&self, ctx: &ReducerContext);

    fn take(&mut self) -> Self {
        std::mem::take(self)
    }
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

pub trait ServerLoader<N> {
    fn load(self, ctx: &ReducerContext) -> Result<N, String>;
}

#[allow(dead_code)]
pub trait NodeExt: Sized + ServerNode + StringData {
    fn get(ctx: &ReducerContext, id: u64) -> Option<Self>;
    fn parent<P: NodeExt>(&self, ctx: &ReducerContext) -> Option<P>;
    fn child<P: NodeExt>(&self, ctx: &ReducerContext) -> Option<P>;
    fn find_parent<P: NodeExt>(&self, ctx: &ReducerContext) -> Option<P>;
    fn find_child<P: NodeExt>(&self, ctx: &ReducerContext) -> Option<P>;
    fn collect_parents<P: NodeExt>(&self, ctx: &ReducerContext) -> Vec<P>;
    fn collect_children<P: NodeExt>(&self, ctx: &ReducerContext) -> Vec<P>;
    fn collect_owner(ctx: &ReducerContext, owner: u64) -> Vec<Self>;
    fn top_parent<P: NodeExt>(&self, ctx: &ReducerContext) -> Option<P>;
    fn top_child<P: NodeExt>(&self, ctx: &ReducerContext) -> Option<P>;
    fn mutual_top_parent<P: NodeExt>(&self, ctx: &ReducerContext) -> Option<P>;
    fn mutual_top_child<P: NodeExt>(&self, ctx: &ReducerContext) -> Option<P>;
}

impl<T> NodeExt for T
where
    T: ServerNode + StringData,
{
    fn get(ctx: &ReducerContext, id: u64) -> Option<Self> {
        let kind = Self::kind_s().to_string();
        let node: TNode = ctx.db.nodes_world().id().find(id)?;
        if node.kind == kind {
            node.to_node().ok()
        } else {
            None
        }
    }
    fn parent<P: NodeExt>(&self, ctx: &ReducerContext) -> Option<P> {
        self.id()
            .get_kind_parent(ctx, P::kind_s())
            .and_then(|id| id.load_node(ctx).ok())
    }
    fn child<P: NodeExt>(&self, ctx: &ReducerContext) -> Option<P> {
        self.id()
            .get_kind_child(ctx, P::kind_s())
            .and_then(|id| id.load_node(ctx).ok())
    }
    fn find_parent<P: NodeExt>(&self, ctx: &ReducerContext) -> Option<P> {
        self.id()
            .find_kind_parent(ctx, P::kind_s())
            .and_then(|id| id.load_node(ctx).ok())
    }
    fn find_child<P: NodeExt>(&self, ctx: &ReducerContext) -> Option<P> {
        self.id()
            .find_kind_child(ctx, P::kind_s())
            .and_then(|id| id.load_node(ctx).ok())
    }
    fn collect_parents<P: NodeExt>(&self, ctx: &ReducerContext) -> Vec<P> {
        self.id()
            .collect_kind_parents(ctx, P::kind_s())
            .to_nodes(ctx)
    }
    fn collect_children<P: NodeExt>(&self, ctx: &ReducerContext) -> Vec<P> {
        self.id()
            .collect_kind_children(ctx, P::kind_s())
            .to_nodes(ctx)
    }
    fn collect_owner(ctx: &ReducerContext, owner: u64) -> Vec<Self> {
        TNode::collect_kind_owner(ctx, Self::kind_s(), owner).to_nodes()
    }
    fn top_parent<P: NodeExt>(&self, ctx: &ReducerContext) -> Option<P> {
        self.id()
            .top_parent(ctx, P::kind_s())
            .and_then(|id| id.load_node(ctx).ok())
    }
    fn top_child<P: NodeExt>(&self, ctx: &ReducerContext) -> Option<P> {
        self.id()
            .top_child(ctx, P::kind_s())
            .and_then(|id| id.load_node(ctx).ok())
    }
    fn mutual_top_parent<P: NodeExt>(&self, ctx: &ReducerContext) -> Option<P> {
        self.id()
            .mutual_top_parent(ctx, P::kind_s())
            .and_then(|id| id.load_node(ctx).ok())
    }
    fn mutual_top_child<P: NodeExt>(&self, ctx: &ReducerContext) -> Option<P> {
        self.id()
            .mutual_top_child(ctx, P::kind_s())
            .and_then(|id| id.load_node(ctx).ok())
    }
}
