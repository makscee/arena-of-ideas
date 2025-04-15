use include_dir::{DirEntry, File};
use macro_client::*;
use serde::{
    de::{self, Visitor},
    ser::SerializeTuple,
};
use std::fmt::Debug;

macro_schema::nodes!();

#[bevy_trait_query::queryable]
pub trait GetVar: GetNodeKind + Debug {
    fn get_own_var(&self, var: VarName) -> Option<VarValue>;
    fn get_var(&self, var: VarName, context: &Context) -> Option<VarValue>;
    fn get_own_vars(&self) -> Vec<(VarName, VarValue)>;
    fn get_vars(&self, context: &Context) -> Vec<(VarName, VarValue)>;
    fn set_var(&mut self, var: VarName, value: VarValue);
}

pub trait Node: Default + Component + Sized + GetVar + Show + Debug + Hash {
    fn id(&self) -> u64;
    fn get_id(&self) -> Option<u64>;
    fn set_id(&mut self, id: u64);
    fn reassign_ids(&mut self, next_id: &mut u64);
    fn parent(&self) -> u64;
    fn get_parent(&self) -> Option<u64>;
    fn set_parent(&mut self, id: u64);
    fn entity(&self) -> Entity;
    fn get_entity(&self) -> Option<Entity>;
    fn from_dir(parent: u64, path: String, dir: &Dir) -> Option<Self>;
    fn to_dir<'a>(&self, path: String) -> &'a [DirEntry<'a>];
    fn from_tnodes(id: u64, nodes: &Vec<TNode>) -> Option<Self>;
    fn to_tnodes(&self) -> Vec<TNode>;
    fn load_recursive(id: u64) -> Option<Self>;
    fn pack(entity: Entity, context: &Context) -> Option<Self>;
    fn unpack(self, entity: Entity, world: &mut World);
    fn find_up_entity<T: Component>(entity: Entity, world: &World) -> Option<&T> {
        let r = world.get::<T>(entity);
        if r.is_some() {
            r
        } else {
            if let Some(p) = world.get::<Parent>(entity) {
                Self::find_up_entity(p.get(), world)
            } else {
                None
            }
        }
    }
    fn find_up<'a, T: Component>(&self, context: &'a Context) -> Option<&'a T> {
        let entity = self.get_entity().expect("Node not linked to world");
        context.find_parent_component::<T>(entity)
    }
    fn collect_children_entity<'a, T: Component>(entity: Entity, world: &'a World) -> Vec<&'a T> {
        entity
            .get_children(world)
            .into_iter()
            .filter_map(|e| world.get::<T>(e))
            .collect_vec()
    }
    fn collect_children<'a, T: Component>(&self, world: &'a World) -> Vec<&'a T> {
        let entity = self.get_entity().expect("Node not linked to world");
        Self::collect_children_entity(entity, world)
    }
    fn component_kinds() -> HashSet<NodeKind>;
    fn children_kinds() -> HashSet<NodeKind>;
    fn fill_from_incubator(self) -> Self;
    fn clear_ids(&mut self);
    fn with_components(self, context: &Context) -> Self;
    fn egui_id(&self) -> Id {
        if let Some(id) = self.get_id() {
            Id::new(id)
        } else if let Some(entity) = self.get_entity() {
            Id::new(entity)
        } else {
            Id::new(self)
        }
    }
}

pub trait NodeExt: Sized + Node + GetNodeKind + GetNodeKindSelf {
    fn view_id(&self) -> Id {
        Id::new(self.get_entity())
            .with(self.get_id())
            .with(self.kind())
    }
    fn to_tnode(&self) -> TNode;
    fn get<'a>(entity: Entity, context: &'a Context) -> Option<&'a Self>;
    fn get_by_id<'a>(id: u64, context: &'a Context) -> Option<&'a Self>;
    fn load(id: u64) -> Option<Self>;
    fn load_by_parent(parent: u64) -> Option<Self>;
    fn find_incubator_component<T: Node + GetNodeKind + GetNodeKindSelf>(&self) -> Option<T>;
    fn collect_incubator_children<T: Node + GetNodeKind + GetNodeKindSelf>(&self) -> Vec<T>;
}
impl<T> NodeExt for T
where
    T: Node + GetNodeKind + GetNodeKindSelf,
{
    fn to_tnode(&self) -> TNode {
        TNode {
            id: self.id(),
            parent: self.get_parent().unwrap_or_default(),
            kind: self.kind().to_string(),
            data: self.get_data(),
        }
    }
    fn get<'a>(entity: Entity, context: &'a Context) -> Option<&'a Self> {
        context.get_component::<Self>(entity)
    }
    fn get_by_id<'a>(id: u64, context: &'a Context) -> Option<&'a Self> {
        context.get_component_by_id::<Self>(id)
    }
    fn load(id: u64) -> Option<Self> {
        cn().db.nodes_world().id().find(&id)?.to_node().ok()
    }
    fn load_by_parent(parent: u64) -> Option<Self> {
        let kind = Self::kind_s().to_string();
        cn().db
            .nodes_world()
            .iter()
            .find(|n| n.kind == kind && n.parent == parent)
            .and_then(|n| n.to_node().ok())
    }
    fn find_incubator_component<P: Node + GetNodeKind + GetNodeKindSelf>(&self) -> Option<P> {
        let kind = P::kind_s().to_string();
        let id = cn()
            .db
            .incubator_links()
            .iter()
            .filter(|n| n.from == self.id() && n.to_kind == kind)
            .max_by_key(|n| n.score)?
            .to;
        P::load(id)
    }
    fn collect_incubator_children<P: Node + GetNodeKind + GetNodeKindSelf>(&self) -> Vec<P> {
        let kind = self.kind().to_string();
        let child_kind = P::kind_s().to_string();
        let mut candidates = cn()
            .db
            .incubator_links()
            .iter()
            .filter(|l| l.from_kind == child_kind && l.to_kind == kind)
            .map(|l| l.from)
            .unique()
            .collect_vec();
        candidates.retain(|id| {
            cn().db
                .incubator_links()
                .iter()
                .filter(|l| l.from == *id && l.to_kind == kind)
                .max_by_key(|l| l.score)
                .unwrap()
                .to
                == self.id()
        });
        candidates
            .into_iter()
            .filter_map(|id| P::load(id))
            .collect()
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
        d.set_parent(self.parent);
        Ok(d)
    }
    pub fn unpack(&self, entity: Entity, world: &mut World) {
        self.kind().unpack(entity, self, world);
    }
    pub fn to_ron(self) -> String {
        ron::to_string(&SerdeWrapper::new(self)).unwrap()
    }
}

#[derive(Resource, Default)]
pub struct IdEntityLinks {
    map: HashMap<u64, Entity>,
}

pub trait WorldNodeExt {
    fn add_id_link(&mut self, id: u64, entity: Entity);
    fn get_id_link(&self, id: u64) -> Option<Entity>;
    fn clear_id_link(&mut self, id: u64);
}

impl WorldNodeExt for World {
    fn add_id_link(&mut self, id: u64, entity: Entity) {
        self.get_resource_or_insert_with::<IdEntityLinks>(|| default())
            .map
            .insert(id, entity);
    }
    fn get_id_link(&self, id: u64) -> Option<Entity> {
        self.get_resource::<IdEntityLinks>()
            .and_then(|r| r.map.get(&id))
            .copied()
    }
    fn clear_id_link(&mut self, id: u64) {
        if let Some(mut r) = self.get_resource_mut::<IdEntityLinks>() {
            r.map.remove(&id);
        }
    }
}

impl ToCstr for NodeKind {
    fn cstr(&self) -> Cstr {
        self.to_string()
    }
}

impl NodeKind {
    fn on_unpack(self, entity: Entity, world: &mut World) {
        let vars = self.get_vars(entity, world);
        let mut emut = world.entity_mut(entity);
        let mut ns = if let Some(ns) = emut.get_mut::<NodeState>() {
            ns
        } else {
            emut.insert(NodeState::default())
                .get_mut::<NodeState>()
                .unwrap()
        };
        ns.init_vars(vars);
        match self {
            NodeKind::House => {
                ns.init(VarName::visible, false.into());
            }
            _ => {}
        };
        emut.insert((Transform::default(), Visibility::default()));

        let mut child = || world.spawn_empty().set_parent(entity).id();
        match self {
            NodeKind::Fusion => {
                unit_rep().clone().unpack(entity, world);
                NodeState::from_world_mut(entity, world).unwrap().init_vars(
                    [
                        (VarName::pwr, 0.into()),
                        (VarName::hp, 1.into()),
                        (VarName::dmg, 0.into()),
                    ]
                    .into(),
                );
            }
            NodeKind::StatusMagic => status_rep().clone().unpack(child(), world),
            _ => {}
        }
    }
}

impl Team {
    pub fn roster_units_load<'a>(&'a self, context: &'a Context) -> Vec<&'a Unit> {
        self.houses_load(context)
            .into_iter()
            .flat_map(|h| h.units_load(context))
            .collect_vec()
    }
}

impl Unit {
    pub fn to_house(self, context: &Context) -> Result<House, ExpressionError> {
        let mut house = self
            .find_up::<House>(context)
            .cloned()
            .to_e("House not found")?
            .with_components(context);
        house.units.push(self.with_components(context));
        Ok(house)
    }
}

pub trait TableNodeView<T> {
    fn add_node_view_columns(self, kind: NodeKind, f: fn(&T) -> u64) -> Self;
}

impl<'a, T: 'static + Clone + Send + Sync> TableNodeView<T> for Table<'a, T> {
    fn add_node_view_columns(self, kind: NodeKind, f: fn(&T) -> u64) -> Self {
        match kind {
            NodeKind::House => self.column_cstr_opt_dyn("name", move |d, world| {
                House::get_by_id(f(d), &world.into()).map(|n| n.house_name.cstr_s(CstrStyle::Bold))
            }),
            NodeKind::HouseColor => self.column_cstr_opt_dyn("color", move |d, world| {
                HouseColor::get_by_id(f(d), &world.into()).map(|n| {
                    let c = &n.color;
                    format!("[{c} {c}]")
                })
            }),
            NodeKind::AbilityMagic => self.column_cstr_opt_dyn("name", move |d, world| {
                Some(
                    AbilityMagic::get_by_id(f(d), &Context::new(world))?
                        .ability_name
                        .cstr_s(CstrStyle::Bold),
                )
            }),
            NodeKind::AbilityDescription => {
                self.column_cstr_opt_dyn("description", move |d, world| {
                    Some(
                        AbilityDescription::get_by_id(f(d), &world.into())?
                            .description
                            .cstr_s(CstrStyle::Bold),
                    )
                })
            }
            NodeKind::AbilityEffect => {
                self.per_row_render()
                    .column_ui_dyn("data", move |d, _, ui, world| {
                        if let Some(n) = AbilityEffect::get_by_id(f(d), &world.into()) {
                            n.view(ViewContext::new(ui), &default(), ui);
                        }
                    })
            }
            NodeKind::StatusMagic => self.column_cstr_opt_dyn("name", move |d, world| {
                Some(
                    StatusMagic::get_by_id(f(d), &world.into())?
                        .status_name
                        .cstr_s(CstrStyle::Bold),
                )
            }),
            NodeKind::StatusDescription => {
                self.column_cstr_opt_dyn("description", move |d, world| {
                    Some(
                        StatusDescription::get_by_id(f(d), &world.into())?
                            .description
                            .cstr_s(CstrStyle::Bold),
                    )
                })
            }
            NodeKind::Unit => self.column_cstr_opt_dyn("name", move |d, world| {
                Some(
                    Unit::get_by_id(f(d), &world.into())?
                        .unit_name
                        .cstr_s(CstrStyle::Bold),
                )
            }),
            NodeKind::UnitDescription => {
                self.column_cstr_opt_dyn("description", move |d, world| {
                    Some(
                        UnitDescription::get_by_id(f(d), &world.into())?
                            .description
                            .cstr_s(CstrStyle::Bold),
                    )
                })
            }
            NodeKind::UnitStats => self
                .column_cstr_value_dyn(
                    "pwr",
                    move |d, world| {
                        UnitStats::get_by_id(f(d), &world.into())
                            .map(|n| n.pwr.into())
                            .unwrap_or_default()
                    },
                    move |_, value| value.get_i32().unwrap().cstr_c(YELLOW),
                )
                .column_cstr_value_dyn(
                    "hp",
                    move |d, world| {
                        UnitStats::get_by_id(f(d), &world.into())
                            .map(|n| n.hp.into())
                            .unwrap_or_default()
                    },
                    move |_, value| value.get_i32().unwrap().cstr_c(RED),
                )
                .column_cstr_value_dyn(
                    "dmg",
                    move |d, world| {
                        UnitStats::get_by_id(f(d), &world.into())
                            .map(|n| n.dmg.into())
                            .unwrap_or_default()
                    },
                    move |_, value| value.get_i32().unwrap().cstr_c(DARK_RED),
                ),
            NodeKind::Behavior => {
                self.per_row_render()
                    .column_ui_dyn("data", move |d, _, ui, world| {
                        if let Some(n) = Behavior::get_by_id(f(d), &world.into()) {
                            n.view(ViewContext::new(ui), &default(), ui);
                        }
                    })
            }
            NodeKind::Representation => self.row_height(100.0).column_dyn(
                "view",
                |_, _| default(),
                move |d, _, ui, world| {
                    let context = &world.into();
                    if let Some(d) = Representation::get_by_id(f(d), context) {
                        let size = ui.available_height();
                        let (rect, _) =
                            ui.allocate_exact_size(egui::vec2(size, size), Sense::hover());
                        ui.set_clip_rect(ui.clip_rect().intersect(rect));
                        d.paint(rect, context.clone().set_owner(d.entity()), ui)
                            .log();
                    }
                },
                false,
            ),
            _ => unimplemented!(),
        }
    }
}

pub fn core<'a>(context: &'a Context) -> &'a Core {
    Core::get_by_id(ID_CORE, context).unwrap()
}

pub fn node_selector<T: Node + NodeExt + DataView>(ui: &mut Ui, context: &Context) -> Option<T> {
    let resp = format!("add [b {}]", T::kind_s()).cstr().button(ui);
    let mut result = None;
    resp.bar_menu(|ui| {
        if "empty".cstr().button(ui).clicked() {
            result = Some(T::default());
            ui.close_menu();
        }
        let mut show_node = |node: &T, view_ctx, context: &Context, ui: &mut Ui| {
            ui.horizontal(|ui| {
                if "add".cstr().button(ui).clicked() {
                    result = T::pack(node.entity(), context);
                    ui.close_menu();
                }
                node.view(view_ctx, context, ui);
            });
        };
        ui.menu_button("core", |ui| {
            ScrollArea::vertical()
                .min_scrolled_height(500.0)
                .show(ui, |ui| {
                    // for n in world.query::<&T>().iter(world) {
                    //     if n.get_parent().is_none_or(|parent| parent == ID_INCUBATOR) {
                    //         continue;
                    //     }
                    //     show_node(n, ViewContext::new(ui), &default(), ui);
                    // }
                });
        });
        ui.menu_button("incubator", |ui| {
            ScrollArea::vertical()
                .min_scrolled_height(500.0)
                .show(ui, |ui| {
                    // for n in world.query::<&T>().iter(world) {
                    //     if n.parent() != ID_INCUBATOR {
                    //         continue;
                    //     }
                    //     show_node(n, ViewContext::new(ui), &default(), ui);
                    // }
                });
        });
    });
    result
}
