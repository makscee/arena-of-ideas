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

pub trait NodeDomainExt {
    fn insert(self, ctx: &ReducerContext, id: u64, kind: NodeKind, data: String);
    fn find_by_key(self, ctx: &ReducerContext, key: &String) -> Option<TNode>;
    fn filter_by_kind(self, ctx: &ReducerContext, kind: NodeKind) -> Vec<TNode>;
}
impl NodeDomainExt for NodeDomain {
    fn insert(self, ctx: &ReducerContext, id: u64, kind: NodeKind, data: String) {
        match self {
            NodeDomain::World => ctx.db.nodes_world().insert(TNode::new(id, kind, data)),
            NodeDomain::Match => ctx.db.nodes_match().insert(TNode::new(id, kind, data)),
            NodeDomain::Alpha => ctx.db.nodes_alpha().insert(TNode::new(id, kind, data)),
        };
    }
    fn find_by_key(self, ctx: &ReducerContext, key: &String) -> Option<TNode> {
        match self {
            NodeDomain::World => ctx.db.nodes_world().key().find(key),
            NodeDomain::Match => ctx.db.nodes_match().key().find(key),
            NodeDomain::Alpha => ctx.db.nodes_alpha().key().find(key),
        }
    }
    fn filter_by_kind(self, ctx: &ReducerContext, kind: NodeKind) -> Vec<TNode> {
        match self {
            NodeDomain::World => ctx.db.nodes_world().kind().filter(kind.as_ref()).collect(),
            NodeDomain::Match => ctx.db.nodes_match().kind().filter(kind.as_ref()).collect(),
            NodeDomain::Alpha => ctx.db.nodes_alpha().kind().filter(kind.as_ref()).collect(),
        }
    }
}

impl TNode {
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
