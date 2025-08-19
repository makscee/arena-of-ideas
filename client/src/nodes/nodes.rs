use include_dir::{Dir, DirEntry};

use super::*;
use serde::{
    de::{self, Visitor},
    ser::SerializeTuple,
};
use spacetimedb_sats::serde::SerdeWrapper;
use std::fmt::Debug;

include!(concat!(env!("OUT_DIR"), "/client_impls.rs"));

pub trait GetVar: Debug {
    fn get_own_var(&self, var: VarName) -> Option<VarValue>;
    fn get_var(&self, var: VarName, context: &Context) -> Option<VarValue>;
    fn get_own_vars(&self) -> Vec<(VarName, VarValue)>;
    fn get_vars(&self, context: &Context) -> Vec<(VarName, VarValue)>;
    fn set_var(&mut self, var: VarName, value: VarValue);
}

pub trait Node:
    Default + Component + Sized + GetVar + Show + Debug + std::hash::Hash + StringData + Clone + ToCstr
{
    fn id(&self) -> u64;
    fn set_id(&mut self, id: u64);
    fn reassign_ids(&mut self, next_id: &mut u64);
    fn owner(&self) -> u64;
    fn set_owner(&mut self, owner: u64);
    fn entity(&self) -> Entity;
    fn get_entity(&self) -> Option<Entity>;
    fn set_entity(&mut self, entity: Entity);
    fn from_dir(path: String, dir: &Dir) -> Option<Self>;
    fn to_dir<'a>(&self, path: String) -> &'a [DirEntry<'a>];
    fn pack_fill(&self, pn: &mut PackedNodes);
    fn pack(&self) -> PackedNodes;
    fn unpack_id(id: u64, pn: &PackedNodes) -> Option<Self>;
    fn load_recursive(world: &World, id: u64) -> Option<Self>;
    fn pack_entity(context: &Context, entity: Entity) -> Result<Self, ExpressionError>;
    fn unpack_entity(self, context: &mut Context, entity: Entity) -> Result<(), ExpressionError>;
    fn all_linked_parents() -> HashSet<NodeKind>;
    fn all_linked_children() -> HashSet<NodeKind>;
    fn egui_id(&self) -> Id {
        Id::new(self.id())
    }
    fn kind(&self) -> NodeKind {
        NodeKind::from_str(type_name_of_val_short(self)).unwrap()
    }
    fn kind_s() -> NodeKind {
        NodeKind::from_str(type_name_short::<Self>()).unwrap()
    }
}

pub trait NodeExt: Sized + Node {
    fn view_id(&self) -> Id {
        Id::new(self.get_entity()).with(self.id()).with(self.kind())
    }
    fn to_tnode(&self) -> TNode;
    fn get<'a>(entity: Entity, context: &'a Context) -> Result<&'a Self, ExpressionError>;
    fn get_by_id<'a>(id: u64, context: &'a Context) -> Result<&'a Self, ExpressionError>;
    fn load(id: u64) -> Option<Self>;
}
impl<T> NodeExt for T
where
    T: Node + StringData,
{
    fn to_tnode(&self) -> TNode {
        TNode {
            id: self.id(),
            owner: self.owner(),
            kind: self.kind().to_string(),
            data: self.get_data(),
            rating: 0,
        }
    }
    fn get<'a>(entity: Entity, context: &'a Context) -> Result<&'a Self, ExpressionError> {
        context.get::<Self>(entity)
    }
    fn get_by_id<'a>(id: u64, context: &'a Context) -> Result<&'a Self, ExpressionError> {
        context.get::<Self>(context.entity(id)?)
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
    pub fn to_node<T: Node + StringData>(&self) -> Result<T, ExpressionError> {
        let mut d = T::default();
        d.inject_data(&self.data)?;
        d.set_id(self.id);
        d.set_owner(self.owner);
        Ok(d)
    }
    pub fn unpack(&self, context: &mut Context, entity: Entity) {
        self.kind().unpack(context, entity, self);
    }
    pub fn to_ron(self) -> String {
        ron::to_string(&SerdeWrapper::new(self)).unwrap()
    }
}

pub trait NodeKindOnUnpack {
    fn on_unpack(self, context: &mut Context, entity: Entity) -> Result<(), ExpressionError>;
}

impl NodeKindOnUnpack for NodeKind {
    fn on_unpack(self, context: &mut Context, entity: Entity) -> Result<(), ExpressionError> {
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
                context.get_mut::<NodeState>(entity)?.init_vars(
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
    pub fn color_for_text(&self, context: &Context) -> Color32 {
        self.color_load(context)
            .map(|c| c.color.c32())
            .unwrap_or_else(|_| colorix().low_contrast_text())
    }
}

pub trait TableNodeView<T> {
    fn add_node_view_columns(self, kind: NodeKind, f: fn(&T) -> u64) -> Self;
}

pub fn node_menu<T: Node + NodeExt + ViewFns>(ui: &mut Ui, context: &Context) -> Option<T> {
    let mut result = None;
    fn show_node_list<T: Node + NodeExt + ViewFns>(
        context: &Context,
        ui: &mut Ui,
        ids: Vec<u64>,
    ) -> Option<T> {
        let mut result: Option<T> = None;
        ScrollArea::vertical()
            .min_scrolled_height(500.0)
            .show(ui, |ui| {
                let vctx = ViewContext::new(ui);
                let Ok(nodes) = context.collect_components::<T>(ids) else {
                    return;
                };
                for d in nodes {
                    let name = d.title_cstr(vctx, context);
                    if ui
                        .menu_button(name.widget(1.0, ui.style()), |ui| {
                            ScrollArea::both().show(ui, |ui| {
                                d.view(vctx, context, ui);
                            });
                        })
                        .response
                        .clicked()
                    {
                        result = T::pack_entity(context, d.entity()).ok();
                        ui.close_menu();
                    }
                }
            });
        result
    }
    ui.menu_button("core", |ui| {
        result = show_node_list(
            context,
            ui,
            cn().db
                .nodes_world()
                .iter()
                .filter_map(|n| if n.owner == ID_CORE { Some(n.id) } else { None })
                .collect_vec(),
        );
    });
    ui.menu_button("all", |ui| {
        result = show_node_list(
            context,
            ui,
            cn().db
                .nodes_world()
                .iter()
                .filter_map(|n| if n.owner == 0 { Some(n.id) } else { None })
                .collect_vec(),
        );
    });
    result
}
pub fn new_node_btn<T: Node + NodeExt + ViewFns>(ui: &mut Ui) -> Option<T> {
    if format!("add [b {}]", T::kind_s())
        .cstr()
        .button(ui)
        .clicked()
    {
        let n = T::default();
        Some(n)
    } else {
        None
    }
}
