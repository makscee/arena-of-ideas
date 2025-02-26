use std::collections::HashSet;

use macro_server::*;
use schema::*;
use strum_macros::{Display, EnumIter};

macro_schema::nodes!();

pub trait Node: Default + Sized {
    fn id(&self) -> u64;
    fn get_id(&self) -> Option<u64>;
    fn set_id(&mut self, id: u64);
    fn clear_ids(&mut self);
    fn gather_ids(&self, data: &mut HashSet<u64>);
    fn inject_data(&mut self, data: &str);
    fn get_data(&self) -> String;
    fn from_strings(i: usize, strings: &Vec<String>) -> Option<Self>;
    fn to_strings(&self, parent: usize, field: &str, strings: &mut Vec<String>);
    fn with_components(self, ctx: &ReducerContext) -> Self;
    fn with_children(self, ctx: &ReducerContext) -> Self;
    fn save(self, ctx: &ReducerContext);
}

pub trait NodeExt: Sized {
    fn to_tnode(&self, id: u64) -> TNode;
    fn get(ctx: &ReducerContext, id: u64) -> Option<Self>;
    fn get_by_data(ctx: &ReducerContext, data: &String) -> Option<Self>;
    fn find_parent(ctx: &ReducerContext, id: u64) -> Option<Self>;
    fn set_parent(&self, ctx: &ReducerContext, parent: u64);
    fn insert(&self, ctx: &ReducerContext);
    fn update(&self, ctx: &ReducerContext);
    fn delete(&self, ctx: &ReducerContext);
    fn insert_or_update(&self, ctx: &ReducerContext);
    fn tnode_collect_kind(ctx: &ReducerContext, kind: NodeKind) -> Vec<TNode>;
    fn collect_kind(ctx: &ReducerContext) -> Vec<Self>;
}

impl<T> NodeExt for T
where
    T: Node + GetNodeKind + GetNodeKindSelf,
{
    fn to_tnode(&self, id: u64) -> TNode {
        TNode::new(id, self.kind(), self.get_data())
    }
    fn get(ctx: &ReducerContext, id: u64) -> Option<Self> {
        let kind = Self::kind_s();
        ctx.db
            .tnodes()
            .key()
            .find(kind.key(id))
            .map(|d| d.to_node())
    }
    fn get_by_data(ctx: &ReducerContext, data: &String) -> Option<T> {
        let kind = T::kind_s().to_string();
        let node = ctx.db.tnodes().data().find(data)?;
        if node.key != kind {
            None
        } else {
            Some(node.to_node())
        }
    }
    fn insert(&self, ctx: &ReducerContext) {
        let node = self.to_tnode(self.id());
        ctx.db.tnodes().insert(node);
    }
    fn update(&self, ctx: &ReducerContext) {
        let node = self.to_tnode(self.id());
        ctx.db.tnodes().key().update(node);
    }
    fn delete(&self, ctx: &ReducerContext) {
        let key = self.kind().key(self.id());
        ctx.db.tnodes().key().delete(key);
    }
    fn insert_or_update(&self, ctx: &ReducerContext) {
        self.delete(ctx);
        self.insert(ctx);
    }
    fn tnode_collect_kind(ctx: &ReducerContext, kind: NodeKind) -> Vec<TNode> {
        ctx.db.tnodes().kind().filter(&kind.to_string()).collect()
    }
    fn collect_kind(ctx: &ReducerContext) -> Vec<Self> {
        Self::tnode_collect_kind(ctx, T::kind_s())
            .into_iter()
            .map(|d| d.to_node::<T>())
            .collect()
    }
    fn find_parent(ctx: &ReducerContext, id: u64) -> Option<Self> {
        let mut id = id;
        while let Some(parent) = id.parent(ctx) {
            id = parent;
            if let Some(node) = T::get(ctx, id) {
                return Some(node);
            }
        }
        None
    }
    fn set_parent(&self, ctx: &ReducerContext, parent: u64) {
        ctx.db.nodes_relations().insert(TNodeRelation {
            id: self.id(),
            parent,
        });
    }
}
