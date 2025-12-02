use super::*;
use std::collections::{HashSet, VecDeque};

#[table(public, name = nodes_world,
    index(name = kind_owner, btree(columns = [kind, owner])),
    index(name = kind_data_owner, btree(columns = [kind, data, owner])))]
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

#[spacetimedb::table(name = votes, public)]
pub struct TVotes {
    #[primary_key]
    pub player_id: u64,
    pub upvotes: i32,
    pub downvotes: i32,
}

#[spacetimedb::table(name = votes_history, public)]
pub struct TVotesHistory {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    #[index(btree)]
    pub player_id: u64,
    #[index(btree)]
    pub node_id: u64,
    pub is_upvote: bool,
    pub timestamp: u64,
}

#[spacetimedb::table(name = creators, public)]
pub struct TCreators {
    #[primary_key]
    pub node_id: u64,
    #[index(btree)]
    pub player_id: u64,
}

#[spacetimedb::table(name = creation_phases, public)]
pub struct TCreationPhases {
    #[primary_key]
    pub node_id: u64,
    pub fixed_kinds: Vec<String>,
}

#[table(public, name = node_links,
    index(name = parent_child, btree(columns = [parent, child])),
    index(name = parent_child_kind, btree(columns = [parent, child_kind])),
    index(name = child_parent_kind, btree(columns = [child, parent_kind])))]
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
}

impl TVotes {
    pub fn get_or_create(ctx: &ReducerContext, player_id: u64) -> Self {
        ctx.db
            .votes()
            .player_id()
            .find(player_id)
            .unwrap_or_else(|| {
                ctx.db.votes().insert(Self {
                    player_id,
                    upvotes: 0,
                    downvotes: 0,
                })
            })
    }

    pub fn add_votes(ctx: &ReducerContext, player_id: u64, amount: i32) {
        let mut votes = Self::get_or_create(ctx, player_id);
        votes.upvotes += amount;
        votes.downvotes += amount;
        ctx.db.votes().player_id().update(votes);
    }

    fn vote_node(
        ctx: &ReducerContext,
        player_id: u64,
        node_id: u64,
        is_upvote: bool,
    ) -> NodeResult<()> {
        if false
            && ctx
                .db
                .votes_history()
                .iter()
                .any(|h| h.player_id == player_id && h.node_id == node_id)
        {
            return Err("Already voted on this node".into());
        }

        // Check if player has votes available
        let mut votes = Self::get_or_create(ctx, player_id);
        let available_votes = if is_upvote {
            votes.upvotes
        } else {
            votes.downvotes
        };

        if available_votes <= 0 {
            let vote_type = if is_upvote { "upvotes" } else { "downvotes" };
            return Err(format!("No {} available", vote_type).into());
        }

        // Deduct vote from player
        if is_upvote {
            votes.upvotes -= 1;
        } else {
            votes.downvotes -= 1;
        }
        ctx.db.votes().player_id().update(votes);

        // Record vote in history
        ctx.db.votes_history().insert(TVotesHistory {
            id: 0,
            player_id,
            node_id,
            is_upvote,
            timestamp: ctx.timestamp.to_micros_since_unix_epoch() as u64,
        });
        let mut node = ctx.db.nodes_world().id().find(node_id).to_not_found()?;
        let old_rating = node.rating;
        node.rating += if is_upvote { 1 } else { -1 };
        ctx.db.nodes_world().id().update(node.clone());

        if node.rating == INCUBATOR_VOTES_THRESHOLD {
            ComponentFixer::fix_component(ctx, &node)?;
        } else if old_rating > 0 && node.rating == 0 && !is_upvote {
            ComponentFixer::unfix_component(ctx, &node)?;
        }

        if !is_upvote && node.rating <= -INCUBATOR_VOTES_THRESHOLD {
            TNode::delete_by_id_recursive(ctx, node.id);
        }

        let kind = node.kind();
        if kind.component_children().is_empty() && node.owner == ID_INCUBATOR {
            ComponentFixer::check_base_completion(ctx, &node)?;
        }

        Ok(())
    }

    pub fn upvote_node(ctx: &ReducerContext, player_id: u64, node_id: u64) -> NodeResult<()> {
        Self::vote_node(ctx, player_id, node_id, true)
    }

    pub fn downvote_node(ctx: &ReducerContext, player_id: u64, node_id: u64) -> NodeResult<()> {
        Self::vote_node(ctx, player_id, node_id, false)
    }
}

impl TCreators {
    pub fn record_creation(ctx: &ReducerContext, player_id: u64, node_id: u64) {
        ctx.db.creators().insert(Self { player_id, node_id });
    }

    pub fn get_creator(ctx: &ReducerContext, node_id: u64) -> Option<u64> {
        ctx.db
            .creators()
            .node_id()
            .find(node_id)
            .map(|c| c.player_id)
    }
}

impl TNodeLink {
    pub fn add_by_id(
        ctx: &ReducerContext,
        parent: u64,
        child: u64,
        parent_kind: String,
        child_kind: String,
    ) -> NodeResult<Self> {
        if let Some(link) = ctx
            .db
            .node_links()
            .parent_child()
            .filter((&parent, &child))
            .next()
        {
            return Ok(link);
        }
        Ok(ctx.db.node_links().insert(Self {
            id: 0,
            child,
            parent,
            child_kind,
            parent_kind,
        }))
    }

    pub fn add(ctx: &ReducerContext, child: &TNode, parent: &TNode) -> NodeResult<Self> {
        Self::add_by_id(
            ctx,
            parent.id,
            child.id,
            parent.kind.clone(),
            child.kind.clone(),
        )
    }

    pub fn parents(ctx: &ReducerContext, id: u64) -> Vec<Self> {
        ctx.db.node_links().child().filter(id).collect()
    }

    pub fn children(ctx: &ReducerContext, id: u64) -> Vec<Self> {
        ctx.db.node_links().parent().filter(id).collect()
    }

    pub fn parents_of_kind(ctx: &ReducerContext, id: u64, kind: NodeKind) -> Vec<Self> {
        ctx.db
            .node_links()
            .child_parent_kind()
            .filter((&id, &kind.to_string()))
            .collect()
    }

    pub fn children_of_kind(ctx: &ReducerContext, id: u64, kind: NodeKind) -> Vec<Self> {
        ctx.db
            .node_links()
            .parent_child_kind()
            .filter((&id, &kind.to_string()))
            .collect()
    }

    pub fn update(self, ctx: &ReducerContext) {
        ctx.db.node_links().id().update(self);
    }

    pub fn insert(self, ctx: &ReducerContext) {
        ctx.db.node_links().insert(self);
    }
}

#[allow(unused)]
pub trait NodeIdExt {
    fn load_node<T: Node>(self, ctx: &ReducerContext) -> NodeResult<T>;
    fn load_tnode(self, ctx: &ReducerContext) -> Option<TNode>;
    fn load_tnode_err(self, ctx: &ReducerContext) -> NodeResult<TNode>;
    fn kind(self, ctx: &ReducerContext) -> Option<NodeKind>;
    fn add_parent(self, ctx: &ReducerContext, id: u64) -> NodeResult<()>;
    fn add_child(self, ctx: &ReducerContext, id: u64) -> NodeResult<()>;
    fn remove_parent(self, ctx: &ReducerContext, id: u64);
    fn remove_child(self, ctx: &ReducerContext, id: u64);
    fn get_kind_parent(self, ctx: &ReducerContext, kind: NodeKind) -> Option<u64>;
    fn get_kind_child(self, ctx: &ReducerContext, kind: NodeKind) -> Option<u64>;
    fn find_kind_parent(self, ctx: &ReducerContext, kind: NodeKind) -> Option<u64>;
    fn find_kind_child(self, ctx: &ReducerContext, kind: NodeKind) -> Option<u64>;
    fn collect_kind_parents(self, ctx: &ReducerContext, kind: NodeKind) -> Vec<u64>;
    fn collect_kind_children(self, ctx: &ReducerContext, kind: NodeKind) -> Vec<u64>;
    fn collect_parents_recursive(self, ctx: &ReducerContext) -> HashSet<u64>;
    fn collect_children_recursive(self, ctx: &ReducerContext) -> HashSet<u64>;
    fn collect_parents(self, ctx: &ReducerContext) -> HashSet<u64>;
    fn collect_children(self, ctx: &ReducerContext) -> HashSet<u64>;
    fn has_parent(self, ctx: &ReducerContext, id: u64) -> bool;
    fn has_child(self, ctx: &ReducerContext, id: u64) -> bool;
    fn fixed_kinds(self, ctx: &ReducerContext) -> HashSet<NodeKind>;
}

impl NodeIdExt for u64 {
    fn load_node<T: Node>(self, ctx: &ReducerContext) -> NodeResult<T> {
        self.load_tnode(ctx)
            .ok_or("Node not found".into())
            .and_then(|t| t.to_node())
    }

    fn load_tnode(self, ctx: &ReducerContext) -> Option<TNode> {
        ctx.db.nodes_world().id().find(self)
    }

    fn load_tnode_err(self, ctx: &ReducerContext) -> NodeResult<TNode> {
        self.load_tnode(ctx).ok_or("Node not found".into())
    }

    fn kind(self, ctx: &ReducerContext) -> Option<NodeKind> {
        ctx.db.nodes_world().id().find(self).map(|v| v.kind())
    }

    fn add_parent(self, ctx: &ReducerContext, parent: u64) -> NodeResult<()> {
        let child = TNode::load(ctx, self).ok_or("Link child not found")?;
        let parent = TNode::load(ctx, parent).ok_or("Link parent not found")?;
        TNodeLink::add(ctx, &child, &parent)?;
        Ok(())
    }

    fn add_child(self, ctx: &ReducerContext, child: u64) -> NodeResult<()> {
        let parent = TNode::load(ctx, self).ok_or("Link parent not found")?;
        let child = TNode::load(ctx, child).ok_or("Link child not found")?;
        TNodeLink::add(ctx, &child, &parent)?;
        Ok(())
    }

    fn remove_parent(self, ctx: &ReducerContext, id: u64) {
        let links: Vec<_> = ctx
            .db
            .node_links()
            .parent_child()
            .filter((id, self))
            .collect();
        for l in links {
            ctx.db.node_links().id().delete(l.id);
        }
    }

    fn remove_child(self, ctx: &ReducerContext, id: u64) {
        let links: Vec<_> = ctx
            .db
            .node_links()
            .parent_child()
            .filter((self, id))
            .collect();
        for l in links {
            ctx.db.node_links().id().delete(l.id);
        }
    }

    fn get_kind_parent(self, ctx: &ReducerContext, kind: NodeKind) -> Option<u64> {
        // Get parent with highest rating
        TNodeLink::parents_of_kind(ctx, self, kind)
            .into_iter()
            .filter_map(|l| ctx.db.nodes_world().id().find(l.parent))
            .max_by_key(|n| n.rating)
            .map(|n| n.id)
    }

    fn get_kind_child(self, ctx: &ReducerContext, kind: NodeKind) -> Option<u64> {
        // Get child with highest rating
        TNodeLink::children_of_kind(ctx, self, kind)
            .into_iter()
            .filter_map(|l| ctx.db.nodes_world().id().find(l.child))
            .max_by_key(|n| n.rating)
            .map(|n| n.id)
    }

    fn find_kind_parent(self, ctx: &ReducerContext, kind: NodeKind) -> Option<u64> {
        let mut checked: HashSet<u64> = HashSet::new();
        let mut q = VecDeque::from([self]);
        let kind = kind.to_string();
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
        let mut checked: HashSet<u64> = HashSet::new();
        let mut q = VecDeque::from([self]);
        let kind = kind.to_string();
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
        TNodeLink::parents_of_kind(ctx, self, kind)
            .into_iter()
            .map(|l| l.parent)
            .collect()
    }

    fn collect_kind_children(self, ctx: &ReducerContext, kind: NodeKind) -> Vec<u64> {
        TNodeLink::children_of_kind(ctx, self, kind)
            .into_iter()
            .map(|l| l.child)
            .collect()
    }

    fn collect_parents(self, ctx: &ReducerContext) -> HashSet<u64> {
        HashSet::from_iter(TNodeLink::parents(ctx, self).into_iter().map(|l| l.parent))
    }

    fn collect_children(self, ctx: &ReducerContext) -> HashSet<u64> {
        HashSet::from_iter(TNodeLink::children(ctx, self).into_iter().map(|l| l.child))
    }

    fn collect_parents_recursive(self, ctx: &ReducerContext) -> HashSet<u64> {
        let mut result: HashSet<u64> = HashSet::new();
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
        let mut result: HashSet<u64> = HashSet::new();
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

    fn has_parent(self, ctx: &ReducerContext, id: u64) -> bool {
        ctx.db
            .node_links()
            .parent_child()
            .filter((id, self))
            .next()
            .is_some()
    }

    fn has_child(self, ctx: &ReducerContext, id: u64) -> bool {
        ctx.db
            .node_links()
            .parent_child()
            .filter((self, id))
            .next()
            .is_some()
    }

    fn fixed_kinds(self, ctx: &ReducerContext) -> HashSet<NodeKind> {
        HashSet::from_iter(
            ctx.db
                .creation_phases()
                .node_id()
                .find(self)
                .map(|cp| cp.fixed_kinds)
                .unwrap_or_default()
                .into_iter()
                .map(|k| k.to_kind()),
        )
    }
}

impl TNode {
    pub fn kind(&self) -> NodeKind {
        NodeKind::from_str(&self.kind).unwrap()
    }

    pub fn load(ctx: &ReducerContext, id: u64) -> Option<Self> {
        ctx.db.nodes_world().id().find(id)
    }

    pub fn delete_by_id(ctx: &ReducerContext, id: u64) {
        ctx.db.nodes_world().id().delete(id);
        ctx.db.node_links().child().delete(id);
        ctx.db.node_links().parent().delete(id);
    }

    pub fn delete_by_id_recursive(ctx: &ReducerContext, id: u64) {
        let mut ids = id.collect_children_recursive(ctx);
        ids.insert(id);
        for id in &ids {
            Self::delete_by_id(ctx, *id);
        }
    }

    pub fn to_node<T: Node>(&self) -> NodeResult<T> {
        let mut d = T::default();
        d.inject_data(&self.data)?;
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
