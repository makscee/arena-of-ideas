use super::*;

use spacetimedb_sats::serde::SerdeWrapper;
use std::fmt::Debug;

include!(concat!(env!("OUT_DIR"), "/client_nodes.rs"));

pub trait ClientNode:
    Default + BevyComponent + Sized + FDisplay + Debug + StringData + Clone + ToCstr + schema::Node
{
    fn spawn(self, ctx: &mut ClientContext, entity: Option<Entity>) -> NodeResult<()>;
    fn entity(&self, ctx: &ClientContext) -> NodeResult<Entity> {
        ctx.entity(self.id())
    }
    fn from_file(path: &str) -> NodeResult<Self> {
        let data = std::fs::read_to_string(path)
            .map_err(|e| NodeError::from(format!("Failed to read file {}: {}", path, e)))?;
        let mut node = Self::default();
        node.inject_data(&data)?;
        Ok(node)
    }
    fn remap_ids(mut self) -> Self {
        let mut next_id = next_id();
        let mut id_map = std::collections::HashMap::new();
        self.reassign_ids(&mut next_id, &mut id_map);
        set_next_id(next_id);
        self
    }
}

pub trait NodeExt: ClientNode {
    fn db_load(id: u64) -> NodeResult<Self> {
        TNode::find(id).to_not_found()?.to_node()
    }
}

impl<T: ClientNode> NodeExt for T {}

impl TNode {
    pub fn find(id: u64) -> Option<Self> {
        cn().db.nodes_world().id().find(&id)
    }
    pub fn kind(&self) -> NodeKind {
        self.kind.to_kind()
    }
    pub fn to_node<T: ClientNode + StringData>(&self) -> NodeResult<T> {
        let mut d = T::default();
        d.inject_data(&self.data)?;
        d.set_id(self.id);
        d.set_owner(self.owner);
        Ok(d)
    }
    pub fn to_ron(self) -> String {
        ron::to_string(&SerdeWrapper::new(self)).unwrap()
    }
}

pub trait NodeKindOnSpawn {
    fn on_spawn(self, context: &mut ClientContext, id: u64) -> NodeResult<()>;
}

impl NodeKindOnSpawn for NodeKind {
    fn on_spawn(self, ctx: &mut ClientContext, id: u64) -> NodeResult<()> {
        let entity = ctx.entity(id)?;
        let vars = node_kind_match!(self, ctx.load::<NodeType>(id)?.get_vars());

        // Only create NodeStateHistory for battle simulations
        if ctx.battle().is_ok() {
            let world = ctx.world_mut()?;
            let mut emut = world.entity_mut(entity);
            let mut ns = if let Some(ns) = emut.get_mut::<NodeStateHistory>() {
                ns
            } else {
                emut.insert(NodeStateHistory::default())
                    .get_mut::<NodeStateHistory>()
                    .unwrap()
            };
            ns.init_vars(vars.into_iter());
        }

        let world = ctx.world_mut()?;
        let mut emut = world.entity_mut(entity);
        if let Some(mut ne) = emut.get_mut::<NodeEntity>() {
            ne.add_node(id, self);
        } else {
            emut.insert(NodeEntity::new(id, self));
        };

        emut.insert((Transform::default(), Visibility::default()));

        match self {
            NodeKind::NFusion => {
                if ctx
                    .get_children_of_kind(id, NodeKind::NUnitRepresentation)?
                    .is_empty()
                {
                    let world = ctx.world_mut()?;
                    let rep_entity = world.spawn_empty().id();
                    unit_rep()
                        .clone()
                        .with_id(next_id())
                        .spawn(ctx, Some(rep_entity))?;
                    ctx.add_link_entities(entity, rep_entity)?;
                }

                let mut fusion = ctx.load::<NFusion>(id)?.clone();
                fusion.recalculate_stats(ctx)?;
                fusion.save(ctx)?;
            }
            _ => {}
        }
        Ok(())
    }
}

impl NHouse {
    pub fn color_for_text(&self, ctx: &ClientContext) -> Color32 {
        self.color_ref(ctx)
            .map(|c| c.color.c32())
            .unwrap_or_else(|_| colorix().low_contrast_text())
    }
}

pub trait TableNodeView<T> {
    fn add_node_view_columns(self, kind: NodeKind, f: fn(&T) -> u64) -> Self;
}
