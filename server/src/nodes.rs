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
    fn save(self, ctx: &ReducerContext, parent: u64);
}

pub trait NodeExt: Sized {
    fn to_tnode(&self, id: u64) -> TNode;
    fn get(c: &ReducerContext, id: u64) -> Option<Self>;
    fn get_by_data(c: &ReducerContext, data: &str) -> Option<Self>;
    fn find_parent(c: &ReducerContext, id: u64) -> Option<Self>;
    fn insert(&self, c: &ReducerContext);
    fn update(&self, c: &ReducerContext);
    fn delete(&self, c: &ReducerContext);
    fn insert_or_update(&self, c: &ReducerContext);
    fn tnode_collect_kind(c: &ReducerContext, kind: NodeKind) -> Vec<TNode>;
    fn collect_kind(c: &ReducerContext) -> Vec<Self>;
}

impl<T> NodeExt for T
where
    T: Node + GetNodeKind + GetNodeKindSelf,
{
    fn to_tnode(&self, id: u64) -> TNode {
        TNode::new(id, self.kind(), self.get_data())
    }
    fn get(c: &ReducerContext, id: u64) -> Option<Self> {
        let kind = Self::kind_s();
        c.db.tnodes().key().find(kind.key(id)).map(|d| d.to_node())
    }
    fn get_by_data(c: &ReducerContext, data: &str) -> Option<T> {
        let kind = T::kind_s().to_string();
        c.db.tnodes()
            .data()
            .filter(data)
            .filter(|n| n.kind == kind)
            .map(|n| n.to_node())
            .next()
    }
    fn insert(&self, c: &ReducerContext) {
        let node = self.to_tnode(self.id());
        c.db.tnodes().insert(node);
    }
    fn update(&self, c: &ReducerContext) {
        let node = self.to_tnode(self.id());
        c.db.tnodes().key().update(node);
    }
    fn delete(&self, c: &ReducerContext) {
        let key = self.kind().key(self.id());
        c.db.tnodes().key().delete(key);
    }
    fn insert_or_update(&self, c: &ReducerContext) {
        self.delete(c);
        self.insert(c);
    }
    fn tnode_collect_kind(c: &ReducerContext, kind: NodeKind) -> Vec<TNode> {
        c.db.tnodes().kind().filter(&kind.to_string()).collect()
    }
    fn collect_kind(c: &ReducerContext) -> Vec<Self> {
        Self::tnode_collect_kind(c, T::kind_s())
            .into_iter()
            .map(|d| d.to_node::<T>())
            .collect()
    }
    fn find_parent(c: &ReducerContext, id: u64) -> Option<Self> {
        let mut id = id;
        while let Some(parent) = id.parent(c) {
            id = parent;
            if let Some(node) = T::get(c, id) {
                return Some(node);
            }
        }
        None
    }
}
