use std::collections::{HashSet, VecDeque};

use super::*;

#[table(public, name = nodes_world)]
#[table(public, name = nodes_match)]
#[table(public, name = nodes_alpha)]
pub struct TNode {
    #[primary_key]
    pub key: String,
    #[index(btree)]
    pub id: u64,
    #[index(btree)]
    pub kind: String,
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
    fn all_descendants(self, ctx: &ReducerContext) -> Vec<u64>;
}
impl NodeIdExt for u64 {
    fn parent(self, ctx: &ReducerContext) -> Option<u64> {
        ctx.db.nodes_relations().id().find(self).map(|n| n.parent)
    }
    fn all_descendants(self, ctx: &ReducerContext) -> Vec<u64> {
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

pub trait NodeDomainExt {
    fn node_get<T: Node + GetNodeKindSelf>(self, ctx: &ReducerContext, id: u64) -> Option<T>;
    fn node_insert(self, ctx: &ReducerContext, node: &(impl Node + GetNodeKind));
    fn node_update(self, ctx: &ReducerContext, node: &(impl Node + GetNodeKind));
    fn node_delete(self, ctx: &ReducerContext, node: &(impl Node + GetNodeKind));
    fn node_insert_or_update(self, ctx: &ReducerContext, node: &(impl Node + GetNodeKind));
    fn node_collect<T: Node + GetNodeKindSelf>(self, ctx: &ReducerContext) -> Vec<T>;
    fn node_parent<T: Node + GetNodeKindSelf>(self, ctx: &ReducerContext, id: u64) -> Option<T>;
    fn tnode_find_by_key(self, ctx: &ReducerContext, key: &String) -> Option<TNode>;
    fn tnode_filter_by_kind(self, ctx: &ReducerContext, kind: NodeKind) -> Vec<TNode>;
    fn delete_by_id(self, ctx: &ReducerContext, id: u64);
    fn tnode_collect_kind(self, ctx: &ReducerContext, kind: NodeKind) -> Vec<TNode>;
}
impl NodeDomainExt for NodeDomain {
    fn node_get<T: Node + GetNodeKindSelf>(self, ctx: &ReducerContext, id: u64) -> Option<T> {
        let kind = T::kind_s();
        match self {
            NodeDomain::World => ctx
                .db
                .nodes_world()
                .key()
                .find(kind.key(id))
                .map(|d| d.to_node()),
            NodeDomain::Match => ctx
                .db
                .nodes_match()
                .key()
                .find(kind.key(id))
                .map(|d| d.to_node()),
            NodeDomain::Alpha => ctx
                .db
                .nodes_alpha()
                .key()
                .find(kind.key(id))
                .map(|d| d.to_node()),
        }
    }
    fn node_insert(self, ctx: &ReducerContext, node: &(impl Node + GetNodeKind)) {
        let node = node.to_tnode(node.id());
        match self {
            NodeDomain::World => ctx.db.nodes_world().insert(node),
            NodeDomain::Match => ctx.db.nodes_match().insert(node),
            NodeDomain::Alpha => ctx.db.nodes_alpha().insert(node),
        };
    }
    fn node_update(self, ctx: &ReducerContext, node: &(impl Node + GetNodeKind)) {
        let node = node.to_tnode(node.id());
        match self {
            NodeDomain::World => ctx.db.nodes_world().key().update(node),
            NodeDomain::Match => ctx.db.nodes_match().key().update(node),
            NodeDomain::Alpha => ctx.db.nodes_alpha().key().update(node),
        };
    }
    fn node_delete(self, ctx: &ReducerContext, node: &(impl Node + GetNodeKind)) {
        let key = node.kind().key(node.id());
        match self {
            NodeDomain::World => ctx.db.nodes_world().key().delete(key),
            NodeDomain::Match => ctx.db.nodes_match().key().delete(key),
            NodeDomain::Alpha => ctx.db.nodes_alpha().key().delete(key),
        };
    }
    fn node_insert_or_update(self, ctx: &ReducerContext, node: &(impl Node + GetNodeKind)) {
        self.node_delete(ctx, node);
        self.node_insert(ctx, node);
    }
    fn tnode_find_by_key(self, ctx: &ReducerContext, key: &String) -> Option<TNode> {
        match self {
            NodeDomain::World => ctx.db.nodes_world().key().find(key),
            NodeDomain::Match => ctx.db.nodes_match().key().find(key),
            NodeDomain::Alpha => ctx.db.nodes_alpha().key().find(key),
        }
    }
    fn tnode_filter_by_kind(self, ctx: &ReducerContext, kind: NodeKind) -> Vec<TNode> {
        match self {
            NodeDomain::World => ctx.db.nodes_world().kind().filter(kind.as_ref()).collect(),
            NodeDomain::Match => ctx.db.nodes_match().kind().filter(kind.as_ref()).collect(),
            NodeDomain::Alpha => ctx.db.nodes_alpha().kind().filter(kind.as_ref()).collect(),
        }
    }
    fn node_collect<T: Node + GetNodeKindSelf>(self, ctx: &ReducerContext) -> Vec<T> {
        self.tnode_collect_kind(ctx, T::kind_s())
            .into_iter()
            .map(|d| d.to_node::<T>())
            .collect()
    }
    fn node_parent<T: Node + GetNodeKindSelf>(self, ctx: &ReducerContext, id: u64) -> Option<T> {
        let kind = T::kind_s();
        let mut id = id;
        while let Some(parent) = id.parent(ctx) {
            id = parent;
            if let Some(node) = self.tnode_find_by_key(ctx, &kind.key(id)) {
                return Some(node.to_node());
            }
        }
        None
    }
    fn tnode_collect_kind(self, ctx: &ReducerContext, kind: NodeKind) -> Vec<TNode> {
        match self {
            NodeDomain::World => ctx
                .db
                .nodes_world()
                .kind()
                .filter(&kind.to_string())
                .collect(),
            NodeDomain::Match => ctx
                .db
                .nodes_match()
                .kind()
                .filter(&kind.to_string())
                .collect(),
            NodeDomain::Alpha => ctx
                .db
                .nodes_alpha()
                .kind()
                .filter(&kind.to_string())
                .collect(),
        }
    }
    fn delete_by_id(self, ctx: &ReducerContext, id: u64) {
        let ids = id.all_descendants(ctx);
        match self {
            NodeDomain::World => {
                for id in &ids {
                    ctx.db.nodes_world().id().delete(id);
                }
            }
            NodeDomain::Match => {
                for id in &ids {
                    ctx.db.nodes_match().id().delete(id);
                }
            }
            NodeDomain::Alpha => {
                for id in &ids {
                    ctx.db.nodes_alpha().id().delete(id);
                }
            }
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
    pub fn gather(ctx: &ReducerContext, id: u64) -> Vec<Self> {
        let mut result: Vec<TNode> = default();
        let mut processed: HashSet<u64> = default();
        let mut queue: VecDeque<u64> = VecDeque::from([id]);
        while let Some(id) = queue.pop_front() {
            processed.insert(id);
            result.extend(ctx.db.nodes_world().id().filter(id));
            for node in ctx.db.nodes_relations().parent().filter(id) {
                let id = node.id;
                if !processed.contains(&id) {
                    queue.push_back(id);
                }
            }
        }
        result
    }
}

trait NodeExt {
    fn insert(&self, ctx: &ReducerContext, id: u64);
    fn update(&self, ctx: &ReducerContext, id: u64);
    fn to_tnode(&self, id: u64) -> TNode;
}

impl<T> NodeExt for T
where
    T: Node + GetNodeKind,
{
    fn insert(&self, ctx: &ReducerContext, id: u64) {
        ctx.db
            .nodes_world()
            .insert(TNode::new(id, self.kind(), self.get_data()));
    }
    fn update(&self, ctx: &ReducerContext, id: u64) {
        ctx.db.nodes_world().key().update(self.to_tnode(id));
    }
    fn to_tnode(&self, id: u64) -> TNode {
        TNode::new(id, self.kind(), self.get_data())
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
        ctx.db.nodes_world().insert(TNode::new(id, kind, data));
    }
    Ok(())
}

#[reducer]
fn node_spawn_hero(ctx: &ReducerContext, name: String) -> Result<(), String> {
    let id = next_id(ctx);
    let hero = Hero::new(name);
    let mover = Mover::new();
    hero.insert(ctx, id);
    mover.insert(ctx, id);
    Ok(())
}

#[reducer]
fn node_move(ctx: &ReducerContext, id: u64, x: f32, y: f32) -> Result<(), String> {
    let key = NodeKind::Mover.key(id);
    let data = ctx
        .db
        .nodes_world()
        .key()
        .find(&key)
        .to_e_s("Mover node not found")?
        .data;
    let mut mover = Mover::default();
    mover.inject_data(&data);
    mover.from = mover.pos(GlobalSettings::get(ctx).hero_speed);
    mover.start_ts = now_seconds();
    mover.target = vec2(x, y);
    mover.update(ctx, id);
    Ok(())
}
