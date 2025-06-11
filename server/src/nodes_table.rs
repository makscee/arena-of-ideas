use std::collections::VecDeque;

use super::*;

#[table(public, name = nodes_world,
    index(name = kind_owner, btree(columns = [kind, owner])),
    index(name = kind_data, btree(columns = [kind, data])))]
#[derive(Clone, Debug)]
pub struct TNode {
    #[primary_key]
    pub id: u64,
    #[index(btree)]
    pub owner: u64,
    #[index(btree)]
    pub kind: String,
    #[index(btree)]
    pub data: String,
    pub rating: i32,
}

#[table(public, name = node_links,
    index(name = parent_child, btree(columns = [parent, child, solid])),
    index(name = parent_child_kind, btree(columns = [parent, child_kind, solid])),
    index(name = child_parent_kind, btree(columns = [child, parent_kind, solid])),
)]
#[derive(Debug)]
pub struct TNodeLink {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    #[index(btree)]
    pub parent: u64,
    #[index(btree)]
    pub child: u64,
    #[index(btree)]
    pub parent_kind: String,
    #[index(btree)]
    pub child_kind: String,
    pub rating: i32,
    pub solid: bool,
}

pub trait TopLink {
    fn top(&self) -> Option<&TNodeLink>;
}

impl TopLink for Vec<TNodeLink> {
    fn top(&self) -> Option<&TNodeLink> {
        self.into_iter().sorted_by_key(|l| -l.rating).next()
    }
}

impl TNodeLink {
    pub fn add_by_id(
        ctx: &ReducerContext,
        parent: u64,
        child: u64,
        parent_kind: String,
        child_kind: String,
        solid: bool,
    ) -> Result<(), String> {
        if ctx
            .db
            .node_links()
            .parent_child()
            .filter((&parent, &child))
            .next()
            .is_some()
        {
            return Err("Link already present".into());
        }
        ctx.db.node_links().insert(Self {
            id: 0,
            child,
            parent,
            child_kind,
            parent_kind,
            rating: 0,
            solid,
        });
        Ok(())
    }
    pub fn add(
        ctx: &ReducerContext,
        child: &TNode,
        parent: &TNode,
        solid: bool,
    ) -> Result<(), String> {
        Self::add_by_id(
            ctx,
            parent.id,
            child.id,
            parent.kind.clone(),
            child.kind.clone(),
            solid,
        )
    }
    pub fn solidify(ctx: &ReducerContext, parent: u64, child: u64) -> Result<(), String> {
        let mut link = ctx
            .db
            .node_links()
            .parent_child()
            .filter((&parent, &child))
            .exactly_one()
            .map_err(|e| e.to_string())?;
        if link.solid {
            return Err("Link is already solid".into());
        }
        link.solid = true;
        ctx.db.node_links().id().update(link);
        Ok(())
    }
    pub fn parents(ctx: &ReducerContext, id: u64) -> Vec<Self> {
        ctx.db.node_links().child().filter(id).collect()
    }
    pub fn children(ctx: &ReducerContext, id: u64) -> Vec<Self> {
        ctx.db.node_links().parent().filter(id).collect()
    }
    pub fn parents_of_kind(
        ctx: &ReducerContext,
        id: u64,
        kind: NodeKind,
        solid: bool,
    ) -> Vec<Self> {
        ctx.db
            .node_links()
            .child_parent_kind()
            .filter((&id, &kind.to_string(), &solid))
            .collect()
    }
    pub fn children_of_kind(
        ctx: &ReducerContext,
        id: u64,
        kind: NodeKind,
        solid: bool,
    ) -> Vec<Self> {
        ctx.db
            .node_links()
            .parent_child_kind()
            .filter((&id, &kind.to_string(), &solid))
            .collect()
    }
    pub fn update(self, ctx: &ReducerContext) {
        ctx.db.node_links().id().update(self);
    }
    pub fn insert(self, ctx: &ReducerContext) {
        ctx.db.node_links().insert(self);
    }
    pub fn vote(ctx: &ReducerContext, parent: &TNode, child: &TNode, vote: bool) {
        let vote = if vote { 1 } else { -1 };
        if let Some(mut link) = ctx
            .db
            .node_links()
            .parent_child()
            .filter((&parent.id, &child.id))
            .next()
        {
            link.rating += vote;
            link.update(ctx);
        } else {
            Self {
                id: 0,
                parent: parent.id,
                child: child.id,
                parent_kind: parent.kind.clone(),
                child_kind: child.kind.clone(),
                rating: vote,
                solid: false,
            }
            .insert(ctx);
        }
    }
}

pub trait NodeIdExt {
    fn to_node<T: NodeExt>(self, ctx: &ReducerContext) -> Result<T, String>;
    fn find(self, ctx: &ReducerContext) -> Option<TNode>;
    fn find_err(self, ctx: &ReducerContext) -> Result<TNode, String>;
    fn kind(self, ctx: &ReducerContext) -> Option<NodeKind>;
    fn add_parent(self, ctx: &ReducerContext, id: u64) -> Result<(), String>;
    fn add_child(self, ctx: &ReducerContext, id: u64) -> Result<(), String>;
    fn remove_parent(self, ctx: &ReducerContext, id: u64) -> Result<(), String>;
    fn remove_child(self, ctx: &ReducerContext, id: u64) -> Result<(), String>;
    fn get_kind_parent(self, ctx: &ReducerContext, kind: NodeKind) -> Option<u64>;
    fn get_kind_child(self, ctx: &ReducerContext, kind: NodeKind) -> Option<u64>;
    fn find_kind_parent(self, ctx: &ReducerContext, kind: NodeKind) -> Option<u64>;
    fn find_kind_child(self, ctx: &ReducerContext, kind: NodeKind) -> Option<u64>;
    fn collect_kind_parents(self, ctx: &ReducerContext, kind: NodeKind) -> Vec<u64>;
    fn collect_kind_children(self, ctx: &ReducerContext, kind: NodeKind) -> Vec<u64>;
    fn collect_parents_recursive(self, ctx: &ReducerContext) -> HashSet<u64>;
    fn collect_children_recursive(self, ctx: &ReducerContext) -> HashSet<u64>;
    fn top_child(self, ctx: &ReducerContext, kind: NodeKind) -> Option<u64>;
    fn top_parent(self, ctx: &ReducerContext, kind: NodeKind) -> Option<u64>;
    fn mutual_top_child(self, ctx: &ReducerContext, kind: NodeKind) -> Option<u64>;
    fn mutual_top_parent(self, ctx: &ReducerContext, kind: NodeKind) -> Option<u64>;
}
impl NodeIdExt for u64 {
    fn to_node<T: NodeExt>(self, ctx: &ReducerContext) -> Result<T, String> {
        self.find(ctx)
            .to_custom_e_s_fn(|| format!("Node#{self} not found"))?
            .to_node()
    }
    fn find(self, ctx: &ReducerContext) -> Option<TNode> {
        ctx.db.nodes_world().id().find(self)
    }
    fn find_err(self, ctx: &ReducerContext) -> Result<TNode, String> {
        self.find(ctx)
            .to_custom_e_s_fn(|| format!("Failed to find TNode#{self}"))
    }
    fn kind(self, ctx: &ReducerContext) -> Option<NodeKind> {
        ctx.db.nodes_world().id().find(self).map(|v| v.kind())
    }
    fn add_parent(self, ctx: &ReducerContext, parent: u64) -> Result<(), String> {
        let child =
            TNode::find(ctx, self).to_custom_e_s_fn(|| format!("Link child#{self} not found"))?;
        let parent = TNode::find(ctx, parent)
            .to_custom_e_s_fn(|| format!("Link parent#{parent} not found"))?;
        TNodeLink::add(ctx, &child, &parent, true)?;
        Ok(())
    }
    fn add_child(self, ctx: &ReducerContext, child: u64) -> Result<(), String> {
        let parent =
            TNode::find(ctx, self).to_custom_e_s_fn(|| format!("Link parent#{self} not found"))?;
        let child =
            TNode::find(ctx, child).to_custom_e_s_fn(|| format!("Link child#{child} not found"))?;
        TNodeLink::add(ctx, &child, &parent, true)?;
        Ok(())
    }
    fn remove_parent(self, ctx: &ReducerContext, id: u64) -> Result<(), String> {
        let l = ctx
            .db
            .node_links()
            .parent_child()
            .filter((id, self))
            .next()
            .to_custom_e_s_fn(|| {
                format!("Failed to remove parent#{id} of #{self}: link not found")
            })?;
        ctx.db.node_links().id().delete(l.id);
        Ok(())
    }
    fn remove_child(self, ctx: &ReducerContext, id: u64) -> Result<(), String> {
        let l = ctx
            .db
            .node_links()
            .parent_child()
            .filter((self, id))
            .next()
            .to_custom_e_s_fn(|| {
                format!("Failed to remove child#{id} of #{self}: link not found")
            })?;
        ctx.db.node_links().id().delete(l.id);
        Ok(())
    }
    fn get_kind_parent(self, ctx: &ReducerContext, kind: NodeKind) -> Option<u64> {
        TNodeLink::parents_of_kind(ctx, self, kind, true)
            .top()
            .map(|l| l.parent)
    }
    fn get_kind_child(self, ctx: &ReducerContext, kind: NodeKind) -> Option<u64> {
        TNodeLink::children_of_kind(ctx, self, kind, true)
            .top()
            .map(|l| l.child)
    }
    fn find_kind_parent(self, ctx: &ReducerContext, kind: NodeKind) -> Option<u64> {
        let mut checked: HashSet<u64> = default();
        let mut q = VecDeque::from([self]);
        let kind = kind.as_ref();
        while let Some(id) = q.pop_front() {
            for link in TNodeLink::parents(ctx, id) {
                if !checked.insert(link.parent) {
                    continue;
                }
                if link.parent_kind == kind {
                    return Some(link.parent);
                }
                q.push_back(link.parent);
            }
        }
        None
    }
    fn find_kind_child(self, ctx: &ReducerContext, kind: NodeKind) -> Option<u64> {
        let mut checked: HashSet<u64> = default();
        let mut q = VecDeque::from([self]);
        let kind = kind.as_ref();
        while let Some(id) = q.pop_front() {
            for link in TNodeLink::children(ctx, id) {
                if !checked.insert(link.child) {
                    continue;
                }
                if link.child_kind == kind {
                    return Some(link.child);
                }
                q.push_back(link.child);
            }
        }
        None
    }
    fn collect_kind_parents(self, ctx: &ReducerContext, kind: NodeKind) -> Vec<u64> {
        TNodeLink::parents_of_kind(ctx, self, kind, true)
            .into_iter()
            .map(|l| l.parent)
            .collect()
    }
    fn collect_kind_children(self, ctx: &ReducerContext, kind: NodeKind) -> Vec<u64> {
        TNodeLink::children_of_kind(ctx, self, kind, true)
            .into_iter()
            .map(|l| l.child)
            .collect()
    }
    fn collect_parents_recursive(self, ctx: &ReducerContext) -> HashSet<u64> {
        let mut result: HashSet<u64> = default();
        let mut q: VecDeque<u64> = VecDeque::from([self]);
        while let Some(id) = q.pop_front() {
            for l in TNodeLink::parents(ctx, id) {
                if result.insert(l.parent) {
                    q.push_back(l.parent);
                }
            }
        }
        result
    }
    fn collect_children_recursive(self, ctx: &ReducerContext) -> HashSet<u64> {
        let mut result: HashSet<u64> = default();
        let mut q: VecDeque<u64> = VecDeque::from([self]);
        while let Some(id) = q.pop_front() {
            for l in TNodeLink::children(ctx, id) {
                if result.insert(l.child) {
                    q.push_back(l.child);
                }
            }
        }
        result
    }
    fn top_parent(self, ctx: &ReducerContext, kind: NodeKind) -> Option<u64> {
        TNodeLink::parents_of_kind(ctx, self, kind, false)
            .top()
            .map(|l| l.parent)
    }
    fn top_child(self, ctx: &ReducerContext, kind: NodeKind) -> Option<u64> {
        TNodeLink::children_of_kind(ctx, self, kind, false)
            .top()
            .map(|l| l.child)
    }
    fn mutual_top_child(self, ctx: &ReducerContext, kind: NodeKind) -> Option<u64> {
        let child = TNodeLink::children_of_kind(ctx, self, kind, false)
            .top()?
            .child;
        let parent = TNodeLink::parents_of_kind(ctx, child, self.kind(ctx)?, false)
            .top()?
            .parent;
        if parent == self {
            Some(child)
        } else {
            None
        }
    }
    fn mutual_top_parent(self, ctx: &ReducerContext, kind: NodeKind) -> Option<u64> {
        let parent = TNodeLink::parents_of_kind(ctx, self, kind, false)
            .top()?
            .parent;
        let child = TNodeLink::children_of_kind(ctx, parent, self.kind(ctx)?, false)
            .top()?
            .child;
        if child == self {
            Some(parent)
        } else {
            None
        }
    }
}

impl TNode {
    pub fn kind(&self) -> NodeKind {
        NodeKind::from_str(&self.kind).unwrap()
    }
    pub fn find(ctx: &ReducerContext, id: u64) -> Option<Self> {
        ctx.db.nodes_world().id().find(id)
    }
    pub fn delete_by_id(ctx: &ReducerContext, id: u64) {
        ctx.db.nodes_world().id().delete(id);
        ctx.db.node_links().child().delete(id);
        ctx.db.node_links().parent().delete(id);
    }
    pub fn delete_by_id_recursive(ctx: &ReducerContext, id: u64) {
        let ids = id.collect_children_recursive(ctx);
        for id in &ids {
            Self::delete_by_id(ctx, *id);
        }
    }
    pub fn to_node<T: Node + StringData>(&self) -> Result<T, String> {
        let mut d = T::default();
        d.inject_data(&self.data).to_str_err()?;
        d.set_id(self.id);
        d.set_owner(self.owner);
        Ok(d)
    }
    pub fn new(id: u64, owner: u64, kind: NodeKind, data: String) -> Self {
        Self {
            id,
            owner,
            kind: kind.to_string(),
            data,
            rating: 0,
        }
    }
    pub fn insert(self, ctx: &ReducerContext) {
        ctx.db.nodes_world().insert(self);
    }
    pub fn update(self, ctx: &ReducerContext) {
        ctx.db.nodes_world().id().update(self);
    }
    pub fn collect_kind_owner(ctx: &ReducerContext, kind: NodeKind, owner: u64) -> Vec<Self> {
        ctx.db
            .nodes_world()
            .kind_owner()
            .filter((kind.as_ref(), owner))
            .collect()
    }
}

pub trait TNodeVecExt {
    fn to_nodes<T: NodeExt>(self) -> Vec<T>;
}

impl TNodeVecExt for Vec<TNode> {
    fn to_nodes<T: NodeExt>(self) -> Vec<T> {
        self.into_iter()
            .filter_map(|n| n.to_node::<T>().ok())
            .collect()
    }
}
pub trait IdVecExt {
    fn to_nodes<T: NodeExt>(self, ctx: &ReducerContext) -> Vec<T>;
}
impl IdVecExt for Vec<u64> {
    fn to_nodes<T: NodeExt>(self, ctx: &ReducerContext) -> Vec<T> {
        self.into_iter()
            .filter_map(|n| n.to_node::<T>(ctx).ok())
            .collect()
    }
}


