use include_dir::{DirEntry, File};
use macro_client::*;
use std::fmt::Debug;

macro_schema::nodes!();

#[bevy_trait_query::queryable]
pub trait GetVar: GetNodeKind + Debug {
    fn get_own_var(&self, var: VarName) -> Option<VarValue>;
    fn get_var(&self, var: VarName, context: &Context) -> Option<VarValue>;
    fn get_own_vars(&self) -> Vec<(VarName, VarValue)>;
    fn get_vars(&self, context: &Context) -> Vec<(VarName, VarValue, NodeKind)>;
    fn set_var(&mut self, var: VarName, value: VarValue);
}

pub trait Node: Default + Component + Sized + GetVar + Show + Debug {
    fn id(&self) -> u64;
    fn get_id(&self) -> Option<u64>;
    fn set_id(&mut self, id: u64);
    fn reassign_ids(&mut self, next_id: &mut u64);
    fn parent(&self) -> u64;
    fn get_parent(&self) -> Option<u64>;
    fn set_parent(&mut self, id: u64);
    fn entity(&self) -> Entity;
    fn get_entity(&self) -> Option<Entity>;
    fn from_dir_new(parent: u64, path: String, dir: &Dir) -> Option<Self>;
    fn from_dir(path: String, dir: &Dir) -> Option<Self>;
    fn to_dir(&self, path: String) -> DirEntry;
    fn from_tnodes(id: u64, nodes: &Vec<TNode>) -> Option<Self>;
    fn to_tnodes(&self) -> Vec<TNode>;
    fn load_recursive(id: u64) -> Option<Self>;
    fn pack(entity: Entity, world: &World) -> Option<Self>;
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
    fn find_up<'a, T: Component>(&self, world: &'a World) -> Option<&'a T> {
        let entity = self.get_entity().expect("Node not linked to world");
        Self::find_up_entity::<T>(entity, world)
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
    fn with_components(self, world: &World) -> Self;
}

pub trait NodeExt: Sized + Node + GetNodeKind + GetNodeKindSelf {
    fn view_id(&self) -> Id {
        Id::new(self.get_entity())
            .with(self.get_id())
            .with(self.kind())
    }
    fn to_tnode(&self) -> TNode;
    fn get(entity: Entity, world: &World) -> Option<&Self>;
    fn get_by_id(id: u64, world: &World) -> Option<&Self>;
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
    fn get(entity: Entity, world: &World) -> Option<&Self> {
        world.get::<Self>(entity)
    }
    fn get_by_id(id: u64, world: &World) -> Option<&Self> {
        world.get::<Self>(world.get_id_link(id)?)
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

#[derive(Resource, Default)]
pub struct NameEntityLinks {
    map: HashMap<String, Entity>,
}

pub trait WorldNodeExt {
    fn add_id_link(&mut self, id: u64, entity: Entity);
    fn get_id_link(&self, id: u64) -> Option<Entity>;
    fn add_name_link(&mut self, name: String, entity: Entity);
    fn get_name_link(&self, name: &str) -> Option<Entity>;
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
    fn add_name_link(&mut self, name: String, entity: Entity) {
        self.get_resource_or_insert_with::<NameEntityLinks>(|| default())
            .map
            .insert(name, entity);
    }
    fn get_name_link(&self, name: &str) -> Option<Entity> {
        self.get_resource::<NameEntityLinks>()
            .and_then(|r| r.map.get(name))
            .copied()
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
                Fusion::init(entity, world).log();
            }
            NodeKind::StatusAbility => status_rep().clone().unpack(child(), world),
            _ => {}
        }
    }
    fn show_df_name(
        &self,
        entity: Option<Entity>,
        context: &Context,
        ui: &mut Ui,
    ) -> DataFrameResponse {
        let mut response = None;
        match self {
            NodeKind::Unit => {
                if let Some(entity) = entity {
                    if show_unit_tag(context.clone().set_owner(entity), ui).is_ok() {
                        response = Some(ui.allocate_rect(ui.min_rect(), Sense::click()));
                    }
                }
            }
            NodeKind::House => {
                if let Some(entity) = entity {
                    if let Some(house) = context.get_component::<House>(entity) {
                        if let Some(color) = context.get_component::<HouseColor>(entity) {
                            TagWidget::new_name(&house.name, color.color.c32()).ui(ui);
                            response = Some(ui.allocate_rect(ui.min_rect(), Sense::click()));
                        }
                    }
                }
            }
            _ => {}
        }
        if response.is_none() {
            response = Some(self.cstr().button(ui));
        }
        if response.unwrap().clicked() {
            DataFrameResponse::NameClicked
        } else {
            DataFrameResponse::None
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
    pub fn to_house(self, world: &World) -> Result<House, ExpressionError> {
        let mut house = self
            .find_up::<House>(world)
            .cloned()
            .to_e("House not found")?
            .with_components(world);
        house.units.push(self.with_components(world));
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
                House::get_by_id(f(d), world).map(|n| n.name.cstr_s(CstrStyle::Bold))
            }),
            NodeKind::HouseColor => self.column_cstr_opt_dyn("color", move |d, world| {
                HouseColor::get_by_id(f(d), world).map(|n| {
                    let c = &n.color;
                    format!("[{c} {c}]")
                })
            }),
            NodeKind::ActionAbility => self.column_cstr_opt_dyn("name", move |d, world| {
                let n = ActionAbility::get_by_id(f(d), world)?;
                Some(n.name.cstr_s(CstrStyle::Bold))
            }),
            NodeKind::ActionAbilityDescription => {
                self.column_cstr_opt_dyn("description", move |d, world| {
                    let n = ActionAbilityDescription::get_by_id(f(d), world)?;
                    Some(n.description.cstr_s(CstrStyle::Bold))
                })
            }
            NodeKind::AbilityEffect => {
                self.per_row_render()
                    .column_ui_dyn("data", move |d, _, ui, world| {
                        if let Some(n) = AbilityEffect::get_by_id(f(d), world) {
                            n.show(None, &default(), ui);
                        }
                    })
            }
            NodeKind::StatusAbility => self.column_cstr_opt_dyn("name", move |d, world| {
                let n = StatusAbility::get_by_id(f(d), world)?;
                Some(n.name.cstr_s(CstrStyle::Bold))
            }),
            NodeKind::StatusAbilityDescription => {
                self.column_cstr_opt_dyn("description", move |d, world| {
                    let n = StatusAbilityDescription::get_by_id(f(d), world)?;
                    Some(n.description.cstr_s(CstrStyle::Bold))
                })
            }
            NodeKind::Unit => self.column_cstr_opt_dyn("name", move |d, world| {
                let n = Unit::get_by_id(f(d), world)?;
                Some(n.name.cstr_s(CstrStyle::Bold))
            }),
            NodeKind::UnitDescription => {
                self.column_cstr_opt_dyn("description", move |d, world| {
                    let n = UnitDescription::get_by_id(f(d), world)?;
                    Some(n.description.cstr_s(CstrStyle::Bold))
                })
            }
            NodeKind::UnitStats => self
                .column_cstr_value_dyn(
                    "pwr",
                    move |d, world| {
                        UnitStats::get_by_id(f(d), world)
                            .map(|n| n.pwr.into())
                            .unwrap_or_default()
                    },
                    move |_, value| value.get_i32().unwrap().cstr_c(YELLOW),
                )
                .column_cstr_value_dyn(
                    "hp",
                    move |d, world| {
                        UnitStats::get_by_id(f(d), world)
                            .map(|n| n.hp.into())
                            .unwrap_or_default()
                    },
                    move |_, value| value.get_i32().unwrap().cstr_c(RED),
                )
                .column_cstr_value_dyn(
                    "dmg",
                    move |d, world| {
                        UnitStats::get_by_id(f(d), world)
                            .map(|n| n.dmg.into())
                            .unwrap_or_default()
                    },
                    move |_, value| value.get_i32().unwrap().cstr_c(DARK_RED),
                ),
            NodeKind::Behavior => {
                self.per_row_render()
                    .column_ui_dyn("data", move |d, _, ui, world| {
                        if let Some(n) = Behavior::get_by_id(f(d), world) {
                            n.show(None, &default(), ui);
                        }
                    })
            }
            NodeKind::Representation => self.row_height(100.0).column_dyn(
                "view",
                |_, _| default(),
                move |d, _, ui, world| {
                    if let Some(d) = Representation::get_by_id(f(d), world) {
                        let size = ui.available_height();
                        let (rect, _) =
                            ui.allocate_exact_size(egui::vec2(size, size), Sense::hover());
                        ui.set_clip_rect(ui.clip_rect().intersect(rect));
                        d.paint(rect, &Context::new_world(world).set_owner(d.entity()), ui)
                            .log();
                    }
                },
                false,
            ),
            _ => unimplemented!(),
        }
    }
}

pub fn all(world: &World) -> &All {
    All::get_by_id(ID_ALL, world).unwrap()
}
