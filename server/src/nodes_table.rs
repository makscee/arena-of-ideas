use std::collections::VecDeque;

use super::*;

#[table(public, name = tnodes)]
#[derive(Clone, Debug)]
pub struct TNode {
    #[primary_key]
    pub key: String,
    #[index(btree)]
    pub id: u64,
    #[index(btree)]
    pub kind: String,
    #[unique]
    pub data: String,
}

#[table(public, name = nodes_relations)]
pub struct TNodeRelation {
    #[primary_key]
    pub id: u64,
    #[index(btree)]
    pub parent: u64,
}

pub trait NodeIdExt {
    fn parent(self, ctx: &ReducerContext) -> Option<u64>;
    fn children(self, ctx: &ReducerContext) -> Vec<u64>;
    fn children_recursive(self, ctx: &ReducerContext) -> Vec<u64>;
}
impl NodeIdExt for u64 {
    fn parent(self, ctx: &ReducerContext) -> Option<u64> {
        ctx.db.nodes_relations().id().find(self).map(|n| n.parent)
    }
    fn children(self, ctx: &ReducerContext) -> Vec<u64> {
        ctx.db
            .nodes_relations()
            .parent()
            .filter(self)
            .map(|d| d.id)
            .collect()
    }
    fn children_recursive(self, ctx: &ReducerContext) -> Vec<u64> {
        let mut vec: Vec<u64> = default();
        let mut q = VecDeque::from([self]);
        while let Some(id) = q.pop_front() {
            vec.push(id);
            for r in ctx.db.nodes_relations().parent().filter(id) {
                q.push_back(r.id);
            }
        }
        vec
    }
}

impl TNode {
    pub fn find_by_key(ctx: &ReducerContext, key: &String) -> Option<Self> {
        ctx.db.tnodes().key().find(key)
    }
    pub fn filter_by_kind(ctx: &ReducerContext, kind: NodeKind) -> Vec<Self> {
        ctx.db.tnodes().kind().filter(kind.as_ref()).collect()
    }
    pub fn delete_by_id_recursive(ctx: &ReducerContext, id: u64) {
        let ids = id.children_recursive(ctx);
        for id in &ids {
            ctx.db.tnodes().id().delete(id);
        }
        for id in ids {
            ctx.db.nodes_relations().id().delete(id);
        }
    }
}

impl TNode {
    pub fn to_node<T: Node>(self) -> T {
        let mut d = T::default();
        d.inject_data(&self.data);
        d.set_id(self.id);
        d
    }
    pub fn new(id: u64, kind: NodeKind, data: String) -> Self {
        Self {
            key: kind.key(id),
            id,
            kind: kind.to_string(),
            data,
        }
    }
}

#[reducer]
fn node_spawn(
    ctx: &ReducerContext,
    id: Option<u64>,
    kinds: Vec<String>,
    datas: Vec<String>,
) -> Result<(), String> {
    let id = id.unwrap_or_else(|| next_id(ctx));
    for (kind, data) in kinds.into_iter().zip(datas.into_iter()) {
        let kind = NodeKind::from_str(&kind).map_err(|e| e.to_string())?;
        ctx.db.tnodes().insert(TNode::new(id, kind, data));
    }
    Ok(())
}
