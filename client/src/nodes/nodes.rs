use include_dir::{Dir, DirEntry};

use super::*;
use spacetimedb_sats::serde::SerdeWrapper;
use std::fmt::Debug;

include!(concat!(env!("OUT_DIR"), "/client_nodes.rs"));

pub trait ClientNode:
    Default
    + Component
    + Sized
    + FDisplay
    + Debug
    + std::hash::Hash
    + StringData
    + Clone
    + ToCstr
    + schema::Node
{
    fn entity(&self) -> Entity;
    fn get_entity(&self) -> Option<Entity>;
    fn set_entity(&mut self, entity: Entity);
    fn from_dir(path: String, dir: &Dir) -> Option<Self>;
    fn to_dir<'a>(&self, path: String) -> &'a [DirEntry<'a>];
    fn pack_fill(&self, pn: &mut PackedNodes);
    fn pack(&self) -> PackedNodes;
    fn unpack_id(id: u64, pn: &PackedNodes) -> Option<Self>;
    fn load_recursive(world: &World, id: u64) -> Option<Self>;
    fn with_parts(&mut self, context: &ClientContext) -> &mut Self;
    fn pack_entity(
        &mut self,
        context: &ClientContext,
        entity: Entity,
    ) -> Result<Self, ExpressionError>;
    fn unpack_entity(
        self,
        context: &mut ClientContext,
        entity: Entity,
    ) -> Result<(), ExpressionError>;
    fn egui_id(&self) -> Id {
        Id::new(self.id())
    }
}

pub trait NodeExt: Sized + ClientNode + StringData {
    fn view_id(&self) -> Id {
        Id::new(self.get_entity()).with(self.id()).with(self.kind())
    }
    fn to_tnode(&self) -> TNode;
    fn get<'a>(
        entity: Entity,
        context: &'a ClientContext<'a>,
    ) -> Result<&'a Self, ExpressionError> {
        todo!()
    }
    fn get_by_id<'a>(id: u64, context: &'a ClientContext<'a>) -> NodeResult<&'a Self> {
        context.load::<Self>(id)
    }
    fn load(id: u64) -> Option<Self> {
        cn().db.nodes_world().id().find(&id)?.to_node().ok()
    }
}

impl TNode {
    pub fn find(id: u64) -> Option<Self> {
        cn().db.nodes_world().id().find(&id)
    }
    pub fn kind(&self) -> NodeKind {
        self.kind.to_kind()
    }
    pub fn to_node<T: ClientNode + StringData>(&self) -> Result<T, ExpressionError> {
        let mut d = T::default();
        d.inject_data(&self.data)?;
        d.set_id(self.id);
        d.set_owner(self.owner);
        Ok(d)
    }
    pub fn unpack(&self, context: &mut ClientContext, entity: Entity) {
        self.kind().unpack(context, entity, self);
    }
    pub fn to_ron(self) -> String {
        ron::to_string(&SerdeWrapper::new(self)).unwrap()
    }
}

pub trait NodeKindOnUnpack {
    fn on_unpack(self, context: &mut ClientContext, entity: Entity) -> Result<(), ExpressionError>;
}

impl NodeKindOnUnpack for NodeKind {
    fn on_unpack(self, context: &mut ClientContext, entity: Entity) -> Result<(), ExpressionError> {
        let vars = self.get_vars(context, entity);
        let mut emut = context.world_mut()?.entity_mut(entity);
        let mut ns = if let Some(ns) = emut.get_mut::<NodeState>() {
            ns
        } else {
            emut.insert(NodeState::default())
                .get_mut::<NodeState>()
                .unwrap()
        };
        ns.kind = self;
        ns.init_vars(vars);
        match self {
            NodeKind::NUnit => {
                ns.init(VarName::dmg, 0.into());
            }
            _ => {}
        };
        emut.insert((Transform::default(), Visibility::default()));

        match self {
            NodeKind::NFusion => {
                if context
                    .first_child::<NUnitRepresentation>(context.id(entity)?)
                    .is_err()
                {
                    let rep_entity = context.world_mut()?.spawn_empty().id();
                    unit_rep().clone().unpack_entity(context, rep_entity)?;
                    context.link_parent_child_entity(entity, rep_entity)?;
                }
                context.component_mut::<NodeState>(entity)?.init_vars(
                    [
                        (VarName::pwr, 0.into()),
                        (VarName::hp, 0.into()),
                        (VarName::dmg, 0.into()),
                    ]
                    .into(),
                );
            }
            _ => {}
        }
        Ok(())
    }
}

impl NHouse {
    pub fn color_for_text(&self, context: &ClientContext) -> Color32 {
        self.color_load(context)
            .map(|c| c.color.c32())
            .unwrap_or_else(|_| colorix().low_contrast_text())
    }
}

pub trait TableNodeView<T> {
    fn add_node_view_columns(self, kind: NodeKind, f: fn(&T) -> u64) -> Self;
}
