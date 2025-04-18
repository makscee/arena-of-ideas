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
    fn set_id(&mut self, id: u64);
    fn reassign_ids(&mut self, next_id: &mut u64);
    fn parent(&self) -> u64;
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
            parent: self.parent(),
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
        match self {
            NodeKind::NFusion => {
                emut.insert(NFusionStats::default());
            }
            _ => {}
        }
        let mut ns = if let Some(ns) = emut.get_mut::<NodeState>() {
            ns
        } else {
            emut.insert(NodeState::default())
                .get_mut::<NodeState>()
                .unwrap()
        };
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

        let mut child = || world.spawn_empty().set_parent(entity).id();
        match self {
            NodeKind::NFusion => {
                unit_rep().clone().unpack(entity, world);
                NodeState::from_world_mut(entity, world).unwrap().init_vars(
                    [
                        (VarName::pwr, 0.into()),
                        (VarName::hp, 0.into()),
                        (VarName::dmg, 0.into()),
                    ]
                    .into(),
                );
            }
            NodeKind::NStatusMagic => status_rep().clone().unpack(child(), world),
            _ => {}
        }
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

impl NUnit {
    pub fn to_house(self, context: &Context) -> Result<NHouse, ExpressionError> {
        let mut house = self
            .find_up::<NHouse>(context)
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
            NodeKind::NHouse => self.column_cstr_opt_dyn("name", move |d, world| {
                NHouse::get_by_id(f(d), &world.into()).map(|n| n.house_name.cstr_s(CstrStyle::Bold))
            }),
            NodeKind::NHouseColor => self.column_cstr_opt_dyn("color", move |d, world| {
                NHouseColor::get_by_id(f(d), &world.into()).map(|n| {
                    let c = &n.color;
                    format!("[{c} {c}]")
                })
            }),
            NodeKind::NAbilityMagic => self.column_cstr_opt_dyn("name", move |d, world| {
                Some(
                    NAbilityMagic::get_by_id(f(d), &Context::new(world))?
                        .ability_name
                        .cstr_s(CstrStyle::Bold),
                )
            }),
            NodeKind::NAbilityDescription => {
                self.column_cstr_opt_dyn("description", move |d, world| {
                    Some(
                        NAbilityDescription::get_by_id(f(d), &world.into())?
                            .description
                            .cstr_s(CstrStyle::Bold),
                    )
                })
            }
            NodeKind::NAbilityEffect => {
                self.per_row_render()
                    .column_ui_dyn("data", move |d, _, ui, world| {
                        if let Some(n) = NAbilityEffect::get_by_id(f(d), &world.into()) {
                            n.view(ViewContext::new(ui), &default(), ui);
                        }
                    })
            }
            NodeKind::NStatusMagic => self.column_cstr_opt_dyn("name", move |d, world| {
                Some(
                    NStatusMagic::get_by_id(f(d), &world.into())?
                        .status_name
                        .cstr_s(CstrStyle::Bold),
                )
            }),
            NodeKind::NStatusDescription => {
                self.column_cstr_opt_dyn("description", move |d, world| {
                    Some(
                        NStatusDescription::get_by_id(f(d), &world.into())?
                            .description
                            .cstr_s(CstrStyle::Bold),
                    )
                })
            }
            NodeKind::NUnit => self.column_cstr_opt_dyn("name", move |d, world| {
                Some(
                    NUnit::get_by_id(f(d), &world.into())?
                        .unit_name
                        .cstr_s(CstrStyle::Bold),
                )
            }),
            NodeKind::NUnitDescription => {
                self.column_cstr_opt_dyn("description", move |d, world| {
                    Some(
                        NUnitDescription::get_by_id(f(d), &world.into())?
                            .description
                            .cstr_s(CstrStyle::Bold),
                    )
                })
            }
            NodeKind::NUnitStats => self
                .column_cstr_value_dyn(
                    "pwr",
                    move |d, world| {
                        NUnitStats::get_by_id(f(d), &world.into())
                            .map(|n| n.pwr.into())
                            .unwrap_or_default()
                    },
                    move |_, value| value.get_i32().unwrap().cstr_c(YELLOW),
                )
                .column_cstr_value_dyn(
                    "hp",
                    move |d, world| {
                        NUnitStats::get_by_id(f(d), &world.into())
                            .map(|n| n.hp.into())
                            .unwrap_or_default()
                    },
                    move |_, value| value.get_i32().unwrap().cstr_c(RED),
                ),
            NodeKind::NBehavior => {
                self.per_row_render()
                    .column_ui_dyn("data", move |d, _, ui, world| {
                        if let Some(n) = NBehavior::get_by_id(f(d), &world.into()) {
                            n.view(ViewContext::new(ui), &default(), ui);
                        }
                    })
            }
            NodeKind::NRepresentation => self.row_height(100.0).column_dyn(
                "view",
                |_, _| default(),
                move |d, _, ui, world| {
                    let context = &world.into();
                    if let Some(d) = NRepresentation::get_by_id(f(d), context) {
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

pub fn core<'a>(context: &'a Context) -> &'a NCore {
    NCore::get_by_id(ID_CORE, context).unwrap()
}

pub fn node_menu<T: Node + NodeExt + DataView>(ui: &mut Ui, context: &Context) -> Option<T> {
    let mut result = None;
    ui.menu_button("core", |ui| {
        ScrollArea::vertical()
            .min_scrolled_height(500.0)
            .show(ui, |ui| {
                let Some(entity) = context.entity_by_id(ID_CORE) else {
                    return;
                };
                let view_ctx = ViewContext::new(ui);
                for d in context.children_components_recursive::<T>(entity) {
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
                        result = T::pack(d.entity(), context);
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
