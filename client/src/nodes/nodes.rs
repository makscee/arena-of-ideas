use super::*;

use spacetimedb_sats::serde::SerdeWrapper;
use std::fmt::Debug;

include!(concat!(env!("OUT_DIR"), "/client_nodes.rs"));

pub trait ClientNode:
    Default + BevyComponent + Sized + FDisplay + Debug + StringData + Clone + ToCstr + schema::Node
{
    fn spawn(self, ctx: &mut ClientContext, entity: Entity) -> NodeResult<()>;
    fn entity(&self, ctx: &ClientContext) -> NodeResult<Entity> {
        ctx.entity(self.id())
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
    pub fn unpack(&self, ctx: &mut ClientContext, entity: Entity) {
        self.kind().on_unpack(ctx, entity).unwrap();
    }
    pub fn to_ron(self) -> String {
        ron::to_string(&SerdeWrapper::new(self)).unwrap()
    }
}

pub trait NodeKindOnUnpack {
    fn on_unpack(self, context: &mut ClientContext, entity: Entity) -> NodeResult<()>;
}

impl NodeKindOnUnpack for NodeKind {
    fn on_unpack(self, ctx: &mut ClientContext, entity: Entity) -> NodeResult<()> {
        let vars = Vec::new(); // TODO: implement get_vars
        let world = ctx.world_mut()?;
        let mut emut = world.entity_mut(entity);
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
                if ctx
                    .get_children_of_kind(ctx.id(entity)?, NodeKind::NUnitRepresentation)?
                    .is_empty()
                {
                    let world = ctx.world_mut()?;
                    let rep_entity = world.spawn_empty().id();
                    unit_rep().clone().spawn(ctx, rep_entity)?;
                    ctx.add_link_entities(entity, rep_entity)?;
                }
                ctx.world_mut()?
                    .get_mut::<NodeState>(entity)
                    .to_not_found()?
                    .init_vars(
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
        self.color_ref(context)
            .map(|c| c.color.c32())
            .unwrap_or_else(|_| colorix().low_contrast_text())
    }
}

pub trait TableNodeView<T> {
    fn add_node_view_columns(self, kind: NodeKind, f: fn(&T) -> u64) -> Self;
}
