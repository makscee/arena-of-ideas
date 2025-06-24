use super::*;

use schema::*;
use serde::{Deserialize, Serialize};
use serde::{
    de::{self, Visitor},
    ser::SerializeTuple,
};
use strum_macros::{Display, EnumIter};

include!(concat!(env!("OUT_DIR"), "/server_impls.rs"));

pub trait Node: Default + Sized {
    fn id(&self) -> u64;
    fn set_id(&mut self, id: u64);
    fn owner(&self) -> u64;
    fn set_owner(&mut self, id: u64);
    fn reassign_ids(&mut self, next_id: &mut u64);
    fn pack_fill(&self, pn: &mut PackedNodes);
    fn pack(&self) -> PackedNodes;
    fn unpack_id(id: u64, pn: &PackedNodes) -> Option<Self>;
    fn with_components(&mut self, ctx: &ReducerContext) -> &mut Self;
    fn with_children(&mut self, ctx: &ReducerContext) -> &mut Self;
    fn save(&self, ctx: &ReducerContext);
    fn clone_self(&self, ctx: &ReducerContext, owner: u64) -> Self;
    fn clone(&self, ctx: &ReducerContext, owner: u64, remap: &mut HashMap<u64, u64>) -> Self;
    fn component_kinds() -> HashSet<NodeKind>;
    fn children_kinds() -> HashSet<NodeKind>;
    fn collect_ids(&self) -> Vec<u64>;
    fn solidify_links(&self, ctx: &ReducerContext) -> Result<(), String>;
    fn delete_with_components(&self, ctx: &ReducerContext);
    fn kind(&self) -> NodeKind {
        NodeKind::from_str(type_name_of_val_short(self)).unwrap()
    }
    fn kind_s() -> NodeKind {
        NodeKind::from_str(type_name_short::<Self>()).unwrap()
    }
    fn take(&mut self) -> Self {
        std::mem::take(self)
    }
}

pub trait NodeExt: Sized + Node + StringData {
    fn to_tnode(&self) -> TNode;
    fn get(ctx: &ReducerContext, id: u64) -> Option<Self>;
    fn insert_self(&self, ctx: &ReducerContext);
    fn update_self(&self, ctx: &ReducerContext);
    fn delete_self(&self, ctx: &ReducerContext);
    fn parent<P: NodeExt>(&self, ctx: &ReducerContext) -> Option<P>;
    fn child<P: NodeExt>(&self, ctx: &ReducerContext) -> Option<P>;
    fn find_parent<P: NodeExt>(&self, ctx: &ReducerContext) -> Option<P>;
    fn find_child<P: NodeExt>(&self, ctx: &ReducerContext) -> Option<P>;
    fn collect_parents<P: NodeExt>(&self, ctx: &ReducerContext) -> Vec<P>;
    fn collect_children<P: NodeExt>(&self, ctx: &ReducerContext) -> Vec<P>;
    fn collect_owner(ctx: &ReducerContext, owner: u64) -> Vec<Self>;
    fn top_parent<P: NodeExt>(&self, ctx: &ReducerContext) -> Option<P>;
    fn top_child<P: NodeExt>(&self, ctx: &ReducerContext) -> Option<P>;
    fn mutual_top_parent<P: NodeExt>(&self, ctx: &ReducerContext) -> Option<P>;
    fn mutual_top_child<P: NodeExt>(&self, ctx: &ReducerContext) -> Option<P>;
}

impl NFusion {
    fn add(&mut self, ctx: &ReducerContext, id: u64) -> Result<(), String> {
        if self.units.ids.contains(&id) {
            return Err(format!(
                "{}#{} already has parent#{id}",
                self.kind(),
                self.id
            ));
        }
        self.units.ids.push(id);
        self.id.add_parent(ctx, id)?;
        self.update_self(ctx);
        Ok(())
    }
    fn remove(&mut self, ctx: &ReducerContext, id: u64) -> Result<(), String> {
        let Some(i) = self.units.ids.iter().position(|u| *u == id) else {
            return Err(format!(
                "{}#{} does not have parent#{id}",
                self.kind(),
                self.id
            ));
        };
        self.units.ids.remove(i);
        self.id.remove_parent(ctx, id)?;
        self.update_self(ctx);
        Ok(())
    }
}

impl<T> NodeExt for T
where
    T: Node + StringData,
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
    fn parent<P: NodeExt>(&self, ctx: &ReducerContext) -> Option<P> {
        self.id()
            .get_kind_parent(ctx, P::kind_s())
            .and_then(|id| id.to_node(ctx).ok())
    }
    fn child<P: NodeExt>(&self, ctx: &ReducerContext) -> Option<P> {
        self.id()
            .get_kind_child(ctx, P::kind_s())
            .and_then(|id| id.to_node(ctx).ok())
    }
    fn find_parent<P: NodeExt>(&self, ctx: &ReducerContext) -> Option<P> {
        self.id()
            .find_kind_parent(ctx, P::kind_s())
            .and_then(|id| id.to_node(ctx).ok())
    }
    fn find_child<P: NodeExt>(&self, ctx: &ReducerContext) -> Option<P> {
        self.id()
            .find_kind_child(ctx, P::kind_s())
            .and_then(|id| id.to_node(ctx).ok())
    }
    fn collect_parents<P: NodeExt>(&self, ctx: &ReducerContext) -> Vec<P> {
        self.id()
            .collect_kind_parents(ctx, P::kind_s())
            .to_nodes(ctx)
    }
    fn collect_children<P: NodeExt>(&self, ctx: &ReducerContext) -> Vec<P> {
        self.id()
            .collect_kind_children(ctx, P::kind_s())
            .to_nodes(ctx)
    }
    fn collect_owner(ctx: &ReducerContext, owner: u64) -> Vec<Self> {
        TNode::collect_kind_owner(ctx, Self::kind_s(), owner).to_nodes()
    }
    fn top_parent<P: NodeExt>(&self, ctx: &ReducerContext) -> Option<P> {
        self.id()
            .top_parent(ctx, P::kind_s())
            .and_then(|id| id.to_node(ctx).ok())
    }
    fn top_child<P: NodeExt>(&self, ctx: &ReducerContext) -> Option<P> {
        self.id()
            .top_child(ctx, P::kind_s())
            .and_then(|id| id.to_node(ctx).ok())
    }
    fn mutual_top_parent<P: NodeExt>(&self, ctx: &ReducerContext) -> Option<P> {
        self.id()
            .mutual_top_parent(ctx, P::kind_s())
            .and_then(|id| id.to_node(ctx).ok())
    }
    fn mutual_top_child<P: NodeExt>(&self, ctx: &ReducerContext) -> Option<P> {
        self.id()
            .mutual_top_child(ctx, P::kind_s())
            .and_then(|id| id.to_node(ctx).ok())
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
    pub fn clone_ids_remap(&self, ctx: &ReducerContext) -> Result<Self, String> {
        let mut remap: HashMap<u64, u64> = default();
        let mut new_team = self.clone(ctx, self.owner, &mut remap);
        let child_kind = NodeKind::NFusion.to_string();
        let parent_kind = NodeKind::NUnit.to_string();
        for fusion in &mut new_team.fusions {
            fusion.units.ids = fusion
                .units
                .ids
                .iter()
                .map(|u| *remap.get(u).unwrap())
                .collect();
            for id in &fusion.units.ids {
                TNodeLink::add_by_id(
                    ctx,
                    *id,
                    fusion.id,
                    parent_kind.clone(),
                    child_kind.clone(),
                    true,
                )?;
            }
            for (tr, ar) in fusion.behavior.iter_mut() {
                tr.unit = *remap.get(&tr.unit).unwrap();
                for a in ar {
                    a.unit = *remap.get(&a.unit).unwrap();
                }
            }
        }
        new_team.save(ctx);
        Ok(new_team)
    }
}

impl Tier for NBehavior {
    fn tier(&self) -> u8 {
        let action_tiers = self
            .reactions
            .iter()
            .map(|r| r.actions.iter().map(|a| a.tier()).sum::<u8>())
            .sum::<u8>();
        (action_tiers + 1) / 2
    }
}
