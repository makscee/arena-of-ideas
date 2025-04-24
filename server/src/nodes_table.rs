use std::collections::VecDeque;

use super::*;

#[table(public, name = nodes_world)]
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
    pub score: i32,
}

#[table(public, name = node_links, index(name = node_ids, btree(columns = [child, parent])))]
pub struct TNodeLink {
    #[index(btree)]
    pub child: u64,
    #[index(btree)]
    pub parent: u64,
    #[index(btree)]
    pub child_kind: String,
    #[index(btree)]
    pub parent_kind: String,
    pub score: i32,
}

impl TNodeLink {
    pub fn add(ctx: &ReducerContext, child: &TNode, parent: &TNode) -> Result<(), String> {
        ctx.db
            .node_links()
            .try_insert(Self {
                child: child.id,
                parent: parent.id,
                child_kind: child.kind.clone(),
                parent_kind: parent.kind.clone(),
                score: 0,
            })
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    pub fn kind_parents(ctx: &ReducerContext, id: u64, kind: NodeKind) -> Vec<Self> {
        let kind = kind.to_string();
        ctx.db
            .node_links()
            .child()
            .filter(id)
            .filter(|l| l.parent_kind == kind)
            .collect()
    }
    pub fn kind_children(ctx: &ReducerContext, id: u64, kind: NodeKind) -> Vec<Self> {
        let kind = kind.to_string();
        ctx.db
            .node_links()
            .parent()
            .filter(id)
            .filter(|l| l.child_kind == kind)
            .collect()
    }
    pub fn parents(ctx: &ReducerContext, id: u64) -> Vec<Self> {
        ctx.db.node_links().child().filter(id).collect()
    }
    pub fn children(ctx: &ReducerContext, id: u64) -> Vec<Self> {
        ctx.db.node_links().parent().filter(id).collect()
    }
}

pub trait NodeIdExt {
    fn kind(self, ctx: &ReducerContext) -> Option<NodeKind>;
    fn parents(self, ctx: &ReducerContext, id: u64) -> Vec<u64>;
    fn children(self, ctx: &ReducerContext, id: u64) -> Vec<u64>;
    fn add_parent(self, ctx: &ReducerContext, id: u64) -> Result<(), String>;
    fn add_child(self, ctx: &ReducerContext, id: u64) -> Result<(), String>;
    fn get_kind_parent(self, ctx: &ReducerContext, kind: NodeKind) -> Option<u64>;
    fn get_kind_child(self, ctx: &ReducerContext, kind: NodeKind) -> Option<u64>;
    fn find_kind_parent(self, ctx: &ReducerContext, kind: NodeKind) -> Option<u64>;
    fn find_kind_child(self, ctx: &ReducerContext, kind: NodeKind) -> Option<u64>;
    fn collect_kind_parents(self, ctx: &ReducerContext, kind: NodeKind) -> Vec<u64>;
    fn collect_kind_children(self, ctx: &ReducerContext, kind: NodeKind) -> Vec<u64>;
    fn collect_parents_recursive(self, ctx: &ReducerContext) -> HashSet<u64>;
    fn collect_children_recursive(self, ctx: &ReducerContext) -> HashSet<u64>;
}
impl NodeIdExt for u64 {
    fn kind(self, ctx: &ReducerContext) -> Option<NodeKind> {
        ctx.db.nodes_world().id().find(self).map(|v| v.kind())
    }
    fn parents(self, ctx: &ReducerContext, id: u64) -> Vec<u64> {
        TNodeLink::parents(ctx, id)
            .into_iter()
            .map(|l| l.parent)
            .collect()
    }
    fn children(self, ctx: &ReducerContext, id: u64) -> Vec<u64> {
        TNodeLink::children(ctx, id)
            .into_iter()
            .map(|l| l.child)
            .collect()
    }
    fn add_parent(self, ctx: &ReducerContext, parent: u64) -> Result<(), String> {
        let child = TNode::find(ctx, self).to_e_s_fn(|| format!("Link child#{self} not found"))?;
        let parent =
            TNode::find(ctx, parent).to_e_s_fn(|| format!("Link parent#{parent} not found"))?;
        TNodeLink::add(ctx, &child, &parent)?;
        Ok(())
    }
    fn add_child(self, ctx: &ReducerContext, child: u64) -> Result<(), String> {
        let parent =
            TNode::find(ctx, self).to_e_s_fn(|| format!("Link parent#{self} not found"))?;
        let child =
            TNode::find(ctx, child).to_e_s_fn(|| format!("Link child#{child} not found"))?;
        TNodeLink::add(ctx, &child, &parent)?;
        Ok(())
    }
    fn get_kind_parent(self, ctx: &ReducerContext, kind: NodeKind) -> Option<u64> {
        TNodeLink::kind_parents(ctx, self, kind)
            .get(0)
            .map(|l| l.parent)
    }
    fn get_kind_child(self, ctx: &ReducerContext, kind: NodeKind) -> Option<u64> {
        TNodeLink::kind_children(ctx, self, kind)
            .get(0)
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
        TNodeLink::kind_parents(ctx, self, kind)
            .into_iter()
            .map(|l| l.parent)
            .collect()
    }
    fn collect_kind_children(self, ctx: &ReducerContext, kind: NodeKind) -> Vec<u64> {
        TNodeLink::kind_children(ctx, self, kind)
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
        Ok(d)
    }
    pub fn new(id: u64, owner: u64, kind: NodeKind, data: String) -> Self {
        Self {
            id,
            owner,
            kind: kind.to_string(),
            data,
            score: 0,
        }
    }
}

#[reducer]
fn admin_delete_node_recursive(ctx: &ReducerContext, id: u64) -> Result<(), String> {
    ctx.is_admin()?;
    TNode::delete_by_id_recursive(ctx, id);
    Ok(())
}
