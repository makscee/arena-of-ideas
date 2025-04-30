use include_dir::{DirEntry, File};
use macro_client::*;
use serde::{
    de::{self, Visitor},
    ser::SerializeTuple,
};
use std::fmt::Debug;

macro_schema::nodes!();

pub trait GetVar: GetNodeKind + Debug {
    fn get_own_var(&self, var: VarName) -> Option<VarValue>;
    fn get_var(&self, var: VarName, context: &Context) -> Option<VarValue>;
    fn get_own_vars(&self) -> Vec<(VarName, VarValue)>;
    fn get_vars(&self, context: &Context) -> Vec<(VarName, VarValue)>;
    fn set_var(&mut self, var: VarName, value: VarValue);
}

pub trait Node: Default + Component + Sized + GetVar + Show + Debug + Hash {
    fn id(&self) -> u64;
    fn set_id(&mut self, id: u64);
    fn reassign_ids(&mut self, next_id: &mut u64);
    fn owner(&self) -> u64;
    fn set_owner(&mut self, owner: u64);
    fn entity(&self) -> Entity;
    fn get_entity(&self) -> Option<Entity>;
    fn from_dir(path: String, dir: &Dir) -> Option<Self>;
    fn to_dir<'a>(&self, path: String) -> &'a [DirEntry<'a>];
    fn pack_fill(&self, pn: &mut PackedNodes);
    fn pack(&self) -> PackedNodes;
    fn unpack_id(id: u64, pn: &PackedNodes) -> Option<Self>;
    fn load_recursive(world: &World, id: u64) -> Option<Self>;
    fn pack_entity(context: &Context, entity: Entity) -> Result<Self, ExpressionError>;
    fn unpack_entity(self, context: &mut Context, entity: Entity) -> Result<(), ExpressionError>;
    fn component_kinds() -> HashSet<NodeKind>;
    fn children_kinds() -> HashSet<NodeKind>;
    fn with_components(self, context: &Context) -> Self;
    fn egui_id(&self) -> Id {
        Id::new(self.id())
    }
}

pub trait NodeExt: Sized + Node + GetNodeKind + GetNodeKindSelf {
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
    T: Node + GetNodeKind + GetNodeKindSelf,
{
    fn to_tnode(&self) -> TNode {
        TNode {
            id: self.id(),
            owner: self.owner(),
            kind: self.kind().to_string(),
            data: self.get_data(),
            score: 0,
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
    pub fn to_node<T: Node>(&self) -> Result<T, ExpressionError> {
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

impl ToCstr for NodeKind {
    fn cstr(&self) -> Cstr {
        self.to_string()
    }
}

impl NodeKind {
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
            NodeKind::NHouse => {
                ns.init(VarName::visible, false.into());
            }
            NodeKind::NUnit => {
                ns.init(VarName::dmg, 0.into());
            }
            _ => {}
        };
        emut.insert((Transform::default(), Visibility::default()));

        match self {
            NodeKind::NFusion => {
                let rep_entity = context.world_mut()?.spawn_empty().id();
                unit_rep().clone().unpack_entity(context, rep_entity)?;
                context.link_parent_child_entity(entity, rep_entity)?;
                context.get_mut::<NodeState>(entity)?.init_vars(
                    [
                        (VarName::pwr, 0.into()),
                        (VarName::hp, 0.into()),
                        (VarName::dmg, 0.into()),
                    ]
                    .into(),
                );
            }
            NodeKind::NStatusMagic => {
                let rep_entity = context.world_mut()?.spawn_empty().id();
                status_rep().clone().unpack_entity(context, rep_entity)?;
                context.link_parent_child_entity(entity, rep_entity)?;
            }
            _ => {}
        }
        Ok(())
    }
}

impl NTeam {
    pub fn roster_units_load<'a>(&'a self, context: &'a Context) -> Vec<&'a NUnit> {
        self.houses_load(context)
            .into_iter()
            .flat_map(|h| h.units_load(context))
            .collect_vec()
    }
}

pub trait TableNodeView<T> {
    fn add_node_view_columns(self, kind: NodeKind, f: fn(&T) -> u64) -> Self;
}

impl<'a, T: 'static + Clone + Send + Sync> TableNodeView<T> for Table<'a, T> {
    fn add_node_view_columns(self, kind: NodeKind, f: fn(&T) -> u64) -> Self {
        match kind {
            NodeKind::NHouse => self.column_cstr_opt_dyn("name", move |d, world| {
                Context::from_world_ref_r(world, |context| {
                    NHouse::get_by_id(f(d), context).map(|n| n.house_name.cstr_s(CstrStyle::Bold))
                })
                .ok()
            }),
            NodeKind::NHouseColor => self.column_cstr_opt_dyn("color", move |d, world| {
                Context::from_world_ref_r(world, |context| {
                    NHouseColor::get_by_id(f(d), context).map(|n| {
                        let c = &n.color;
                        format!("[{c} {c}]")
                    })
                })
                .ok()
            }),
            NodeKind::NAbilityMagic => self.column_cstr_opt_dyn("name", move |d, world| {
                Context::from_world_ref_r(world, |context| {
                    Ok(NAbilityMagic::get_by_id(f(d), context)?
                        .ability_name
                        .cstr_s(CstrStyle::Bold))
                })
                .ok()
            }),
            NodeKind::NAbilityDescription => {
                self.column_cstr_opt_dyn("description", move |d, world| {
                    Context::from_world_ref_r(world, |context| {
                        Ok(NAbilityDescription::get_by_id(f(d), context)?
                            .description
                            .cstr_s(CstrStyle::Bold))
                    })
                    .ok()
                })
            }
            NodeKind::NAbilityEffect => {
                self.per_row_render()
                    .column_ui_dyn("data", move |d, _, ui, world| {
                        Context::from_world_ref_r(world, |context| {
                            if let Ok(n) = NAbilityEffect::get_by_id(f(d), context) {
                                n.view(ViewContext::new(ui), &default(), ui);
                            }
                            Ok(())
                        })
                        .ui(ui);
                    })
            }
            NodeKind::NStatusMagic => self.column_cstr_opt_dyn("name", move |d, world| {
                Context::from_world_ref_r(world, |context| {
                    Ok(NStatusMagic::get_by_id(f(d), context)?
                        .status_name
                        .cstr_s(CstrStyle::Bold))
                })
                .ok()
            }),
            NodeKind::NStatusDescription => {
                self.column_cstr_opt_dyn("description", move |d, world| {
                    Context::from_world_ref_r(world, |context| {
                        Ok(NStatusDescription::get_by_id(f(d), context)?
                            .description
                            .cstr_s(CstrStyle::Bold))
                    })
                    .ok()
                })
            }
            NodeKind::NUnit => self.column_cstr_opt_dyn("name", move |d, world| {
                Context::from_world_ref_r(world, |context| {
                    Ok(NUnit::get_by_id(f(d), context)?
                        .unit_name
                        .cstr_s(CstrStyle::Bold))
                })
                .ok()
            }),
            NodeKind::NUnitDescription => {
                self.column_cstr_opt_dyn("description", move |d, world| {
                    Context::from_world_ref_r(world, |context| {
                        Ok(NUnitDescription::get_by_id(f(d), context)?
                            .description
                            .cstr_s(CstrStyle::Bold))
                    })
                    .ok()
                })
            }
            NodeKind::NUnitStats => self
                .column_cstr_value_dyn(
                    "pwr",
                    move |d, world| {
                        Context::from_world_ref_r(world, |context| {
                            Ok(NUnitStats::get_by_id(f(d), context)?.pwr)
                        })
                        .unwrap_or_default()
                        .into()
                    },
                    move |_, value| value.get_i32().unwrap().cstr_c(YELLOW),
                )
                .column_cstr_value_dyn(
                    "hp",
                    move |d, world| {
                        Context::from_world_ref_r(world, |context| {
                            Ok(NUnitStats::get_by_id(f(d), context)?.hp)
                        })
                        .unwrap_or_default()
                        .into()
                    },
                    move |_, value| value.get_i32().unwrap().cstr_c(RED),
                ),
            NodeKind::NBehavior => {
                self.per_row_render()
                    .column_ui_dyn("data", move |d, _, ui, world| {
                        Context::from_world_ref_r(world, |context| {
                            if let Ok(n) = NBehavior::get_by_id(f(d), context) {
                                n.view(ViewContext::new(ui), &default(), ui);
                            }
                            Ok(())
                        })
                        .ui(ui);
                    })
            }
            NodeKind::NRepresentation => self.row_height(100.0).column_dyn(
                "view",
                |_, _| default(),
                move |d, _, ui, world| {
                    Context::from_world_ref_r(world, |context| {
                        if let Ok(d) = NRepresentation::get_by_id(f(d), context) {
                            let size = ui.available_height();
                            let (rect, _) =
                                ui.allocate_exact_size(egui::vec2(size, size), Sense::hover());
                            ui.set_clip_rect(ui.clip_rect().intersect(rect));
                            context.with_layer_ref(ContextLayer::Owner(d.entity()), |context| {
                                d.paint(rect, context, ui).ui(ui);
                            });
                        }
                        Ok(())
                    })
                    .ui(ui);
                },
                false,
            ),
            _ => unimplemented!(),
        }
    }
}

pub fn core<'a>(context: &'a Context) -> &'a NCore {
    NCore::get_by_id(ID_CORE, context).unwrap()
}

pub fn node_menu<T: Node + NodeExt + DataView>(ui: &mut Ui, context: &Context) -> Option<T> {
    let mut result = None;
    ui.menu_button("core", |ui| {
        ScrollArea::vertical()
            .min_scrolled_height(500.0)
            .show(ui, |ui| {
                let Ok(entity) = context.entity(ID_CORE) else {
                    return;
                };
                let view_ctx = ViewContext::new(ui);
                for d in entity
                    .id(context)
                    .and_then(|id| context.collect_children_components_recursive::<T>(id))
                    .unwrap_or_default()
                {
                    let name = d.title_cstr(view_ctx, context);
                    if ui
                        .menu_button(name.widget(1.0, ui.style()), |ui| {
                            ScrollArea::both().show(ui, |ui| {
                                d.view(view_ctx, context, ui);
                            });
                        })
                        .response
                        .clicked()
                    {
                        result = T::pack_entity(context, d.entity()).ok();
                        ui.close_menu();
                    }
                }
            })
    });
    result
}

pub fn new_node_btn<T: Node + NodeExt + DataView>(ui: &mut Ui, view_ctx: ViewContext) -> Option<T> {
    if format!("add [b {}]", T::kind_s())
        .cstr()
        .button(ui)
        .clicked()
    {
        let n = T::default();
        view_ctx.merge_state(&n, ui).collapsed(false).save_state(ui);
        Some(n)
    } else {
        None
    }
}
