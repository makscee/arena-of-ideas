use std::collections::HashSet;

use macro_server::*;
use schema::*;
use strum_macros::{Display, EnumIter};

macro_schema::nodes!();

pub trait Node: Default + Sized {
    fn id(&self) -> u64;
    fn parent(&self) -> u64;
    fn set_id(&mut self, id: u64);
    fn from_strings(i: usize, strings: &Vec<String>) -> Option<Self>;
    fn to_strings(&self, parent: usize, field: &str, strings: &mut Vec<String>);
    fn with_components(&mut self, ctx: &ReducerContext) -> &mut Self;
    fn with_children(&mut self, ctx: &ReducerContext) -> &mut Self;
    fn save(self, ctx: &ReducerContext);
    fn clone(&self, ctx: &ReducerContext, parent: u64) -> Self;
}

pub trait NodeExt: Sized + Node + GetNodeKind + GetNodeKindSelf {
    fn to_tnode(&self) -> TNode;
    fn get(ctx: &ReducerContext, id: u64) -> Option<Self>;
    fn find_parent_of_id(ctx: &ReducerContext, id: u64) -> Option<Self>;
    fn find_parent<P: NodeExt>(&self, ctx: &ReducerContext) -> Result<P, String>;
    fn find_child<P: NodeExt>(&self, ctx: &ReducerContext) -> Result<P, String>;
    fn insert_self(&self, ctx: &ReducerContext);
    fn update_self(&self, ctx: &ReducerContext);
    fn delete_self(&self, ctx: &ReducerContext);
    fn delete_recursive(&self, ctx: &ReducerContext);
    fn tnode_collect_kind(ctx: &ReducerContext, kind: NodeKind) -> Vec<TNode>;
    fn collect_kind(ctx: &ReducerContext) -> Vec<Self>;
    fn collect_children_of_id(ctx: &ReducerContext, parent: u64) -> Vec<Self>;
    fn collect_children<P: NodeExt>(&self, ctx: &ReducerContext) -> Vec<P>;
}

impl<T> NodeExt for T
where
    T: Node + GetNodeKind + GetNodeKindSelf + StringData,
{
    fn to_tnode(&self) -> TNode {
        TNode::new(self.id(), self.parent(), self.kind(), self.get_data())
    }
    fn get(ctx: &ReducerContext, id: u64) -> Option<Self> {
        let kind = Self::kind_s().to_string();
        let node: TNode = ctx.db.nodes_world().id().find(id)?;
        if node.kind == kind {
            node.to_node().ok()
        } else {
            None
        }
    }
    fn insert_self(&self, ctx: &ReducerContext) {
        let node = self.to_tnode();
        debug!("insert {node:?}");
        match ctx.db.nodes_world().try_insert(node.clone()) {
            Ok(_) => {}
            Err(e) => error!("Insert of {node:?} failed: {e}"),
        }
    }
    fn update_self(&self, ctx: &ReducerContext) {
        let node = self.to_tnode();
        ctx.db.nodes_world().id().update(node);
    }
    fn delete_self(&self, ctx: &ReducerContext) {
        ctx.db.nodes_world().id().delete(self.id());
    }
    fn delete_recursive(&self, ctx: &ReducerContext) {
        TNode::delete_by_id_recursive(ctx, self.id());
    }
    fn tnode_collect_kind(ctx: &ReducerContext, kind: NodeKind) -> Vec<TNode> {
        ctx.db
            .nodes_world()
            .kind()
            .filter(&kind.to_string())
            .collect()
    }
    fn collect_kind(ctx: &ReducerContext) -> Vec<Self> {
        Self::tnode_collect_kind(ctx, T::kind_s())
            .into_iter()
            .filter_map(|d| d.to_node::<T>().ok())
            .collect()
    }
    fn collect_children_of_id(ctx: &ReducerContext, parent: u64) -> Vec<Self> {
        parent
            .children(ctx)
            .into_iter()
            .filter_map(|id| Self::get(ctx, id))
            .collect()
    }
    fn collect_children<P: NodeExt>(&self, ctx: &ReducerContext) -> Vec<P> {
        P::collect_children_of_id(ctx, self.id())
    }
    fn find_parent_of_id(ctx: &ReducerContext, id: u64) -> Option<Self> {
        let mut id = id;
        while let Some(parent) = id.parent(ctx) {
            id = parent;
            if let Some(node) = T::get(ctx, id) {
                return Some(node);
            }
        }
        None
    }
    fn find_parent<P: NodeExt>(&self, ctx: &ReducerContext) -> Result<P, String> {
        P::find_parent_of_id(ctx, self.id())
            .to_e_s_fn(|| format!("Failed to find parent {}#{}", P::kind_s(), self.id()))
    }
    fn find_child<P: NodeExt>(&self, ctx: &ReducerContext) -> Result<P, String> {
        let mut c = P::collect_children_of_id(ctx, self.id());
        if c.len() > 1 {
            return Err(format!(
                "More than 1 child of {} kind {} found",
                self.id(),
                P::kind_s()
            ));
        }
        if c.is_empty() {
            return Err(format!(
                "No children of {} kind {} found",
                self.id(),
                P::kind_s()
            ));
        }
        Ok(c.remove(0))
    }
}

impl All {
    pub fn load(ctx: &ReducerContext) -> Self {
        All::get(ctx, 0).unwrap()
    }
    pub fn core_units<'a>(&'a mut self, ctx: &ReducerContext) -> Result<Vec<&'a mut Unit>, String> {
        Ok(self
            .core_load(ctx)?
            .into_iter()
            .filter_map(|h| h.units_load(ctx).ok())
            .flatten()
            .collect_vec())
    }
}

impl Match {
    pub fn roster_units_load(&mut self, ctx: &ReducerContext) -> Result<Vec<&mut Unit>, String> {
        Ok(self
            .team_load(ctx)?
            .houses_load(ctx)?
            .into_iter()
            .filter_map(|h| h.units_load(ctx).ok())
            .flatten()
            .collect_vec())
    }
}
