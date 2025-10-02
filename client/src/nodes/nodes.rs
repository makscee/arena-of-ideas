use super::*;

use spacetimedb_sats::serde::SerdeWrapper;
use std::fmt::Debug;

include!(concat!(env!("OUT_DIR"), "/client_nodes.rs"));

pub trait ClientNode:
    Default + BevyComponent + Sized + FDisplay + Debug + StringData + Clone + ToCstr + schema::Node
{
    fn spawn(self, world: &mut World);
}

pub trait NodeExt: Sized + ClientNode + StringData {
    fn get<'a>(entity: Entity, context: &'a ClientContext<'a>) -> Result<&'a Self, NodeError> {
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
    pub fn to_node<T: ClientNode + StringData>(&self) -> NodeResult<T> {
        let mut d = T::default();
        d.inject_data(&self.data)?;
        d.set_id(self.id);
        d.set_owner(self.owner);
        Ok(d)
    }
    pub fn unpack(&self, context: &mut ClientContext, entity: Entity) {
        self.on_unpack(context, entity)?;
    }
    pub fn to_ron(self) -> String {
        ron::to_string(&SerdeWrapper::new(self)).unwrap()
    }
}

pub trait NodeKindOnUnpack {
    fn on_unpack(self, context: &mut ClientContext, entity: Entity) -> NodeResult<()>;
}

impl NodeKindOnUnpack for NodeKind {
    fn on_unpack(self, context: &mut ClientContext, entity: Entity) -> NodeResult<()> {
        let vars = Vec::new(); // TODO: implement get_vars
        let world = context
            .world_mut()
            .to_not_found_msg("World not available")?;
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
                if context
                    .first_child::<NUnitRepresentation>(context.id(entity)?)
                    .is_err()
                {
                    let world = context
                        .world_mut()
                        .to_not_found_msg("World not available")?;
                    let rep_entity = world.spawn_empty().id();
                    unit_rep().clone().unpack_entity(context, rep_entity)?;
                    context.link_parent_child(context.id(entity)?, context.id(rep_entity)?)?;
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
        self.color_ref(context)
            .map(|c| c.color.c32())
            .unwrap_or_else(|_| colorix().low_contrast_text())
    }
}

pub trait TableNodeView<T> {
    fn add_node_view_columns(self, kind: NodeKind, f: fn(&T) -> u64) -> Self;
}
