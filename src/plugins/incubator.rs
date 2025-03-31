use bevy::ecs::event::EventReader;

use super::*;

pub struct IncubatorPlugin;

impl Plugin for IncubatorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<IncubatorData>()
            .add_systems(Startup, Self::startup)
            .add_systems(OnEnter(GameState::Title), Self::init)
            // .add_systems(Update, Self::read_events)
            ;
    }
}

#[derive(Resource, Default)]
struct IncubatorData {
    composed_world: World,
    table_kind: NodeKind,
    inspect_node: Option<(u64, NodeKind)>,
    new_node_link: Option<u64>,
    link_types: Vec<NodeKind>,
    link_type_selected: NodeKind,
    new_node: Option<(NodeKind, Vec<TNode>)>,
}
fn rm(world: &mut World) -> Mut<IncubatorData> {
    world.resource_mut::<IncubatorData>()
}

impl IncubatorPlugin {
    pub fn world_op<T>(world: &mut World, f: impl FnOnce(&mut World) -> T) -> T {
        f(&mut rm(world).composed_world)
    }
    fn read_events(mut events: EventReader<StdbEvent>) {
        if events.is_empty() {
            return;
        }
        if events.read().any(|e| e.node.parent == ID_INCUBATOR) {
            OperationsPlugin::add(|world| {
                Self::compose_nodes(world).log();
            });
        }
    }
    fn startup() {
        on_connect(|_| {
            cn().reducers.on_incubator_push(|e, nodes, _| {
                if !e.check_identity() {
                    return;
                }
                let kind = nodes[0].kind.to_kind();
                e.event.on_success_error(
                    move || {
                        OperationsPlugin::add(move |world| {
                            Self::compose_nodes(world).log();
                            format!("New {kind} added").notify(world);
                            rm(world).new_node = Some((kind, [kind.default_tnode()].into()));
                        });
                    },
                    move || {
                        format!("Failed to add new {kind}").notify_op();
                    },
                );
            });
            cn().reducers.on_incubator_vote(|e, _, _| {
                if !e.check_identity() {
                    return;
                }
                e.event.on_success_error(
                    move || {
                        OperationsPlugin::add(move |world| {
                            Self::compose_nodes(world).log();
                            TableState::reset_cache(&egui_context(world).unwrap());
                            TableState::reset_rows_cache::<(i32, TNode)>(world);
                        });
                    },
                    move || {
                        "Failed to add vote".notify_op();
                    },
                );
            });
        });
    }
    fn init(world: &mut World) {
        let mut r = rm(world);
        r.table_kind = NodeKind::House;
        // Self::compose_nodes(world).log();
    }
    fn compose_nodes(world: &mut World) -> Result<(), ExpressionError> {
        let incubator = Incubator::load_recursive(ID_INCUBATOR).unwrap();
        let houses = incubator
            .houses
            .into_iter()
            .map(|h| h.clone().fill_from_incubator())
            .collect_vec();
        let mut composed_world = World::new();
        dbg!(&houses);
        for house in houses {
            house.unpack(composed_world.spawn_empty().id(), &mut composed_world);
        }
        rm(world).composed_world = composed_world;
        Ok(())
    }
    fn new_node_btn(kind: NodeKind, ui: &mut Ui, world: &mut World) -> bool {
        br(ui);
        if format!("New {kind}")
            .cstr_s(CstrStyle::Bold)
            .button(ui)
            .clicked()
        {
            let mut r = rm(world);
            r.new_node = Some((
                kind,
                [TNode {
                    id: 0,
                    parent: 0,
                    kind: kind.to_string(),
                    data: kind.default_data(),
                }]
                .into(),
            ));
            r.new_node_link = None;
            TilePlugin::set_active(Pane::Incubator(IncubatorPane::NewNode));
            true
        } else {
            false
        }
    }
    pub fn pane_nodes(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        let mut data = rm(world);
        let kind = data.table_kind;
        NodeKind::House.show_graph(CYAN, &mut data, ui);
        ui.vertical(|ui| {
            Self::new_node_btn(kind, ui, world);
            Table::new(kind.to_string(), |_| {
                let kind = kind.to_string();
                cn().db
                    .nodes_world()
                    .iter()
                    .filter(|n| n.kind == kind && n.parent == ID_INCUBATOR)
                    .map(|n| n.id)
                    .collect_vec()
            })
            .add_node_view_columns(kind, |d| *d)
            .add_incubator_columns(kind, |d| *d)
            .ui(ui, world);
            Ok(())
        })
        .inner
    }
    pub fn pane_new_node(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        world.resource_scope(|world, mut d: Mut<IncubatorData>| {
            let (kind, nodes) = if let Some((kind, nodes)) = &mut d.new_node {
                (kind, nodes)
            } else {
                let kind = NodeKind::Unit;
                d.new_node = Some((kind, [kind.default_tnode()].into()));
                let node = d.new_node.as_mut().unwrap();
                (&mut node.0, &mut node.1)
            };
            if Selector::new("Kind").ui_iter(kind, Incubator::children_kinds().iter(), ui) {
                *nodes = [TNode {
                    id: 0,
                    parent: 0,
                    kind: kind.to_string(),
                    data: kind.default_data(),
                }]
                .into();
            }
            match kind {
                NodeKind::House => {
                    if "Add to battle test".cstr().button(ui).clicked() {
                        BattlePlugin::edit_battle(
                            |battle| {
                                if let Some(house) = House::from_tnodes(nodes[0].id, nodes) {
                                    battle.left.houses.retain(|h| h.name != house.name);
                                    battle.left.houses.push(house);
                                }
                            },
                            world,
                        );
                    }
                }
                NodeKind::Unit => {}
                _ => {}
            }
            ui.data_frame_force_open();
            kind.show_tnodes_mut(nodes, ui);
            if "Save".cstr_s(CstrStyle::Bold).button(ui).clicked() {
                cn().reducers
                    .incubator_push(nodes.clone(), d.new_node_link)
                    .unwrap();
            }
        });
        Ok(())
    }
    pub fn pane_inspect(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        let data = rm(world);
        let Some((id, kind)) = data.inspect_node else {
            "Select node to view links"
                .cstr_c(tokens_global().low_contrast_text())
                .label(ui);
            return Ok(());
        };
        format!("{kind} node").cstr_s(CstrStyle::Bold).label(ui);
        if cn()
            .db
            .incubator_nodes()
            .id()
            .find(&id)
            .is_some_and(|n| n.owner == player_id())
        {
            if "Delete Node".cstr().as_button().red(ui).ui(ui).clicked() {
                Confirmation::new("Delete node?")
                    .accept(move |world| {
                        cn().reducers.incubator_delete(id).unwrap();
                        rm(world).inspect_node = None;
                    })
                    .cancel(|_| {})
                    .push(world);
            }
        }
        let data = rm(world);
        match data.composed_world.get_id_link(id) {
            Some(entity) => {
                ui.columns(2, |ui| match kind {
                    NodeKind::Unit => {
                        let context = Context::new_world(&data.composed_world)
                            .set_owner(entity)
                            .take();
                        match UnitCard::from_context(&context) {
                            Ok(c) => {
                                c.show(&context, &mut ui[0]);
                            }
                            Err(e) => {
                                e.cstr().label(&mut ui[0]);
                            }
                        }
                        let ui = &mut ui[1];
                        let size = ui.available_width();
                        let rect = ui
                            .allocate_exact_size(egui::vec2(size, size), Sense::hover())
                            .0
                            .shrink(30.0);
                        if let Some(rep) = data.composed_world.get::<Representation>(entity) {
                            rep.pain_or_show_err(rect, &context, ui);
                        }
                        unit_rep().pain_or_show_err(rect, &context, ui);
                    }
                    _ => {}
                });
                br(ui);
                kind.show(entity, ui, &data.composed_world);
            }
            None => {
                "Node absent in core"
                    .cstr_cs(DARK_RED, CstrStyle::Small)
                    .label(ui);
            }
        }
        br(ui);
        let mut r = rm(world);
        if r.link_types.is_empty() {
            return Ok(());
        }
        let mut selected = r.link_type_selected;
        ui.horizontal(|ui| {
            "show type"
                .cstr_c(tokens_global().low_contrast_text())
                .label(ui);
            if EnumSwitcher::new().show_iter(&mut selected, r.link_types.iter().copied(), ui) {
                r.link_type_selected = selected;
            }
        });
        if Self::new_node_btn(selected, ui, world) {
            let mut r = rm(world);
            r.new_node_link = Some(id);
        }
        Table::new(format!("{selected} links"), move |world| {
            let kind = selected.to_string();
            cn().db
                .nodes_world()
                .iter()
                .filter_map(|n| {
                    if n.parent == ID_INCUBATOR && n.kind == kind {
                        Some((
                            cn().db
                                .incubator_links()
                                .iter()
                                .find_map(|l| {
                                    if l.from == id && l.to_kind == kind && l.to == n.id {
                                        Some(l.score as i32)
                                    } else {
                                        None
                                    }
                                })
                                .unwrap_or_default(),
                            n,
                        ))
                    } else {
                        None
                    }
                })
                .sorted_by_key(|(score, _)| -*score)
                .collect_vec()
        })
        .add_node_view_columns(selected, |(_, node)| node.id)
        .column_int("score", |(score, _)| *score)
        .column_btn_mod_dyn(
            "♥️",
            move |(_, node), _, _| {
                cn().reducers.incubator_vote(id, node.id).unwrap();
            },
            move |(_, node), ui, btn| {
                if let Some(vote) =
                    cn().db
                        .incubator_votes()
                        .key()
                        .find(&vote_key(player_id(), id, selected))
                {
                    btn.active(vote.to == node.id, ui)
                } else {
                    btn
                }
            },
        )
        .add_incubator_columns(selected, |(_, n)| n.id)
        .ui(ui, world);

        Ok(())
    }
}

trait TableIncubatorExt<T> {
    fn add_incubator_columns(self, kind: NodeKind, f: fn(&T) -> u64) -> Self;
}

impl<'a, T: 'static + Clone + Send + Sync> TableIncubatorExt<T> for Table<'a, T> {
    fn add_incubator_columns(mut self, kind: NodeKind, f: fn(&T) -> u64) -> Self {
        self = self
            .add_player_column("owner", move |d, _| {
                let id = f(d);
                cn().db
                    .incubator_nodes()
                    .id()
                    .find(&id)
                    .map(|n| n.owner)
                    .unwrap_or_default()
            })
            .column_btn_dyn("clone", move |d, _, world| {
                let id = f(d);
                let node = cn().db.nodes_world().id().find(&id).unwrap();
                world.resource_mut::<IncubatorData>().new_node = Some((kind, [node].into()));
                TilePlugin::set_active(Pane::Incubator(IncubatorPane::NewNode));
            })
            .column_btn_dyn("inspect", move |d, _, world| {
                let mut r = world.resource_mut::<IncubatorData>();
                r.inspect_node = Some((f(d), kind));
                r.link_types = NodeKind::get_incubator_links()
                    .get(&kind)
                    .unwrap()
                    .iter()
                    .sorted()
                    .copied()
                    .collect();
                if !r.link_types.is_empty() {
                    r.link_type_selected = r.link_types[0];
                }
                TilePlugin::set_active(Pane::Incubator(IncubatorPane::Inspect));
            });
        self
    }
}

fn vote_key(owner: u64, from: u64, kind: NodeKind) -> String {
    format!("{owner}_{from}_{kind}")
}

trait NodeKindGraph {
    fn show_graph(self, color: Color32, data: &mut IncubatorData, ui: &mut Ui);
}

impl NodeKindGraph for NodeKind {
    fn show_graph(self, mut color: Color32, data: &mut IncubatorData, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if let Some((_, inspect)) = data.inspect_node {
                if self == inspect {
                    color = GREEN;
                } else {
                    let links = NodeKind::get_incubator_links();
                    let links = links.get(&inspect).unwrap();
                    if links.contains(&self) {
                        color = PURPLE;
                    }
                }
            }
            if self
                .cstr_cs(color, CstrStyle::Small)
                .as_button()
                .active(data.table_kind == self, ui)
                .ui(ui)
                .clicked()
            {
                data.table_kind = self;
            }
            ui.vertical(|ui| {
                for c in self.component_kinds().into_iter().sorted() {
                    c.show_graph(tokens_global().high_contrast_text(), data, ui);
                }
                for c in self.children_kinds().into_iter().sorted() {
                    c.show_graph(RED, data, ui);
                }
            });
        });
    }
}
