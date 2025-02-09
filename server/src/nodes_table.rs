use std::collections::VecDeque;

use super::*;

#[table(public, name = nodes_world)]
#[table(public, name = nodes_match)]
#[table(public, name = nodes_core)]
pub struct TNode {
    #[primary_key]
    pub key: String,
    #[index(btree)]
    pub id: u64,
    #[index(btree)]
    pub owner: u64,
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
    fn node_get<T: Node + GetNodeKindSelf>(self, c: &Context, id: u64) -> Option<T>;
    fn node_insert(self, c: &Context, node: &(impl Node + GetNodeKind));
    fn node_update(self, c: &Context, node: &(impl Node + GetNodeKind));
    fn node_delete(self, c: &Context, node: &(impl Node + GetNodeKind));
    fn node_insert_or_update(self, c: &Context, node: &(impl Node + GetNodeKind));
    fn node_collect<T: Node + GetNodeKindSelf>(self, c: &Context) -> Vec<T>;
    fn node_parent<T: Node + GetNodeKindSelf>(self, c: &Context, id: u64) -> Option<T>;
    fn tnode_find_by_key(self, c: &Context, key: &String) -> Option<TNode>;
    fn tnode_filter_by_kind(self, c: &Context, kind: NodeKind) -> Vec<TNode>;
    fn delete_by_id(self, c: &Context, id: u64);
    fn tnode_collect_kind(self, c: &Context, kind: NodeKind) -> Vec<TNode>;
    fn tnode_collect_owner(self, c: &Context) -> Vec<TNode>;
}
impl NodeDomainExt for NodeDomain {
    fn node_get<T: Node + GetNodeKindSelf>(self, c: &Context, id: u64) -> Option<T> {
        let kind = T::kind_s();
        match self {
            NodeDomain::World => {
                c.rc.db
                    .nodes_world()
                    .key()
                    .find(kind.key(id))
                    .map(|d| d.to_node())
            }
            NodeDomain::Match => {
                c.rc.db
                    .nodes_match()
                    .key()
                    .find(kind.key(id))
                    .map(|d| d.to_node())
            }
            NodeDomain::Core => {
                c.rc.db
                    .nodes_core()
                    .key()
                    .find(kind.key(id))
                    .map(|d| d.to_node())
            }
        }
    }
    fn node_insert(self, c: &Context, node: &(impl Node + GetNodeKind)) {
        let node = node.to_tnode(node.id(), c.pid());
        match self {
            NodeDomain::World => c.rc.db.nodes_world().insert(node),
            NodeDomain::Match => c.rc.db.nodes_match().insert(node),
            NodeDomain::Core => c.rc.db.nodes_core().insert(node),
        };
    }
    fn node_update(self, c: &Context, node: &(impl Node + GetNodeKind)) {
        let node = node.to_tnode(node.id(), c.pid());
        match self {
            NodeDomain::World => c.rc.db.nodes_world().key().update(node),
            NodeDomain::Match => c.rc.db.nodes_match().key().update(node),
            NodeDomain::Core => c.rc.db.nodes_core().key().update(node),
        };
    }
    fn node_delete(self, c: &Context, node: &(impl Node + GetNodeKind)) {
        let key = node.kind().key(node.id());
        match self {
            NodeDomain::World => c.rc.db.nodes_world().key().delete(key),
            NodeDomain::Match => c.rc.db.nodes_match().key().delete(key),
            NodeDomain::Core => c.rc.db.nodes_core().key().delete(key),
        };
    }
    fn node_insert_or_update(self, c: &Context, node: &(impl Node + GetNodeKind)) {
        self.node_delete(c, node);
        self.node_insert(c, node);
    }
    fn tnode_find_by_key(self, c: &Context, key: &String) -> Option<TNode> {
        match self {
            NodeDomain::World => c.rc.db.nodes_world().key().find(key),
            NodeDomain::Match => c.rc.db.nodes_match().key().find(key),
            NodeDomain::Core => c.rc.db.nodes_core().key().find(key),
        }
    }
    fn tnode_filter_by_kind(self, c: &Context, kind: NodeKind) -> Vec<TNode> {
        match self {
            NodeDomain::World => c.rc.db.nodes_world().kind().filter(kind.as_ref()).collect(),
            NodeDomain::Match => {
                c.rc.db
                    .nodes_match()
                    .kind()
                    .filter(kind.as_ref())
                    .filter(|n| n.owner == c.player.id)
                    .collect()
            }
            NodeDomain::Core => c.rc.db.nodes_core().kind().filter(kind.as_ref()).collect(),
        }
    }
    fn node_collect<T: Node + GetNodeKindSelf>(self, c: &Context) -> Vec<T> {
        self.tnode_collect_kind(c, T::kind_s())
            .into_iter()
            .map(|d| d.to_node::<T>())
            .collect()
    }
    fn node_parent<T: Node + GetNodeKindSelf>(self, c: &Context, id: u64) -> Option<T> {
        let kind = T::kind_s();
        let mut id = id;
        while let Some(parent) = id.parent(c.rc) {
            id = parent;
            if let Some(node) = self.tnode_find_by_key(c, &kind.key(id)) {
                return Some(node.to_node());
            }
        }
        None
    }
    fn tnode_collect_kind(self, c: &Context, kind: NodeKind) -> Vec<TNode> {
        match self {
            NodeDomain::World => {
                c.rc.db
                    .nodes_world()
                    .kind()
                    .filter(&kind.to_string())
                    .collect()
            }
            NodeDomain::Match => {
                c.rc.db
                    .nodes_match()
                    .kind()
                    .filter(&kind.to_string())
                    .filter(|n| n.owner == c.player.id)
                    .collect()
            }
            NodeDomain::Core => {
                c.rc.db
                    .nodes_core()
                    .kind()
                    .filter(&kind.to_string())
                    .collect()
            }
        }
    }
    fn tnode_collect_owner(self, c: &Context) -> Vec<TNode> {
        let owner = c.pid();
        match self {
            NodeDomain::World => c.rc.db.nodes_world().owner().filter(owner).collect_vec(),
            NodeDomain::Match => c.rc.db.nodes_match().owner().filter(owner).collect_vec(),
            NodeDomain::Core => c.rc.db.nodes_core().owner().filter(owner).collect_vec(),
        }
    }
    fn delete_by_id(self, c: &Context, id: u64) {
        let ids = id.all_descendants(c.rc);
        match self {
            NodeDomain::World => {
                for id in &ids {
                    c.rc.db.nodes_world().id().delete(id);
                }
            }
            NodeDomain::Match => {
                for id in &ids {
                    c.rc.db.nodes_match().id().delete(id);
                }
            }
            NodeDomain::Core => {
                for id in &ids {
                    c.rc.db.nodes_core().id().delete(id);
                }
            }
        }
        for id in ids {
            c.rc.db.nodes_relations().id().delete(id);
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
    pub fn new(id: u64, owner: u64, kind: NodeKind, data: String) -> Self {
        Self {
            key: kind.key(id),
            id,
            owner,
            kind: kind.to_string(),
            data,
        }
    }
}

trait NodeExt {
    fn to_tnode(&self, id: u64, owner: u64) -> TNode;
}

impl<T> NodeExt for T
where
    T: Node + GetNodeKind,
{
    fn to_tnode(&self, id: u64, owner: u64) -> TNode {
        TNode::new(id, owner, self.kind(), self.get_data())
    }
}

#[reducer]
fn node_spawn(
    ctx: &ReducerContext,
    id: Option<u64>,
    kinds: Vec<String>,
    datas: Vec<String>,
) -> Result<(), String> {
    let c = Context::new(ctx)?;
    let id = id.unwrap_or_else(|| next_id(ctx));
    for (kind, data) in kinds.into_iter().zip(datas.into_iter()) {
        let kind = NodeKind::from_str(&kind).map_err(|e| e.to_string())?;
        ctx.db
            .nodes_world()
            .insert(TNode::new(id, c.player.id, kind, data));
    }
    Ok(())
}

#[reducer]
fn node_spawn_hero(ctx: &ReducerContext, name: String) -> Result<(), String> {
    let c = &ctx.wrap()?;
    let id = c.next_id();
    let mut hero = Hero::new(name);
    let mut mover = Mover::new();
    hero.id = Some(id);
    mover.id = Some(id);
    NodeDomain::World.node_insert(c, &hero);
    NodeDomain::World.node_insert(c, &mover);
    Ok(())
}

#[reducer]
fn node_move(ctx: &ReducerContext, id: u64, x: f32, y: f32) -> Result<(), String> {
    let c = &ctx.wrap()?;
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
    NodeDomain::World.node_insert_or_update(c, &mover);
    Ok(())
}
