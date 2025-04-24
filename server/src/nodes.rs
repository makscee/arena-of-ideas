use super::*;

use macro_server::*;
use schema::*;
use serde::{
    de::{self, Visitor},
    ser::SerializeTuple,
};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};

macro_schema::nodes!();

pub trait Node: Default + Sized {
    fn id(&self) -> u64;
    fn set_id(&mut self, id: u64);
    fn owner(&self) -> u64;
    fn reassign_ids(&mut self, next_id: &mut u64);
    fn pack_fill(&self, pn: &mut PackedNodes, link: u64);
    fn pack(&self) -> PackedNodes;
    fn unpack_id(id: u64, pn: &PackedNodes) -> Option<Self>;
    fn with_components(&mut self, ctx: &ReducerContext) -> &mut Self;
    fn with_children(&mut self, ctx: &ReducerContext) -> &mut Self;
    fn save(self, ctx: &ReducerContext);
    fn clone_self(&self, ctx: &ReducerContext) -> Self;
    fn clone(&self, ctx: &ReducerContext, remap: &mut HashMap<u64, u64>) -> Self;
    fn component_kinds() -> HashSet<NodeKind>;
    fn children_kinds() -> HashSet<NodeKind>;
    fn take(&mut self) -> Self {
        std::mem::take(self)
    }
}

pub trait NodeExt: Sized + Node + GetNodeKind + GetNodeKindSelf + StringData {
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
        TNode::new(self.id(), self.owner(), self.kind(), self.get_data())
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
        TNode::delete_by_id(ctx, self.id());
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
            .collect_kind_children(ctx, Self::kind_s())
            .into_iter()
            .filter_map(|id| Self::get(ctx, id))
            .sorted_by_key(|n| n.id())
            .collect()
    }
    fn collect_children<P: NodeExt>(&self, ctx: &ReducerContext) -> Vec<P> {
        P::collect_children_of_id(ctx, self.id())
    }
    fn find_parent_of_id(ctx: &ReducerContext, id: u64) -> Option<Self> {
        let kind = Self::kind_s();
        id.find_kind_parent(ctx, kind)
            .and_then(|id| Self::get(ctx, id))
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

impl NCore {
    pub fn load(ctx: &ReducerContext) -> Self {
        NCore::get(ctx, ID_CORE).unwrap()
    }
    pub fn all_units<'a>(&'a mut self, ctx: &ReducerContext) -> Result<Vec<&'a mut NUnit>, String> {
        Ok(self
            .houses_load(ctx)?
            .into_iter()
            .filter_map(|h| h.units_load(ctx).ok())
            .flatten()
            .collect_vec())
    }
}

impl NMatch {
    pub fn roster_units_load(&mut self, ctx: &ReducerContext) -> Result<Vec<&mut NUnit>, String> {
        Ok(self
            .team_load(ctx)?
            .houses_load(ctx)?
            .into_iter()
            .filter_map(|h| h.units_load(ctx).ok())
            .flatten()
            .collect_vec())
    }
}

impl NTeam {
    #[must_use]
    pub fn clone_ids_remap(&self, ctx: &ReducerContext, parent: u64) -> Result<Self, String> {
        let mut remap: HashMap<u64, u64> = default();
        let mut new_team = self.clone(ctx, &mut remap);
        for fusion in &mut new_team.fusions {
            for unit in &mut fusion.units {
                if let Some(id) = remap.get(unit) {
                    *unit = *id;
                }
            }
            fusion.update_self(ctx);
        }
        new_team.id.add_parent(ctx, parent)?;
        Ok(new_team)
    }
}
