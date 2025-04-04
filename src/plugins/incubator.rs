use bevy::ecs::event::EventReader;

use super::*;

pub struct IncubatorPlugin;

impl Plugin for IncubatorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<IncubatorData>()
            .add_systems(Startup, Self::startup)
            .add_systems(OnEnter(GameState::Incubator), Self::init)
            .add_systems(
                Update,
                (Self::update, Self::read_events).run_if(in_state(GameState::Incubator)),
            );
    }
}

#[derive(Resource, Default)]
struct IncubatorData {
    composed_world: World,
    compose_requested: Option<f64>,
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
    fn update(world: &mut World) {
        if rm(world)
            .compose_requested
            .is_some_and(|ts| ts + 0.5 < gt().elapsed())
        {
            Self::compose_nodes(world).log();
            rm(world).compose_requested = None;
        }
    }
    fn read_events(mut events: EventReader<StdbEvent>) {
        if events.is_empty() {
            return;
        }
        if events.read().any(|e| e.node.parent == ID_INCUBATOR) {
            OperationsPlugin::add(|world| {
                Self::compose_request(world);
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
                            Self::compose_request(world);
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
                            Self::compose_request(world);
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
        Self::compose_request(world);
    }
    fn compose_request(world: &mut World) {
        let cr = &mut rm(world).compose_requested;
        if cr.is_none() {
            *cr = Some(gt().elapsed());
        }
    }
    fn compose_nodes(world: &mut World) -> Result<(), ExpressionError> {
        let incubator = Incubator::load_recursive(ID_INCUBATOR).unwrap();
        let houses = incubator
            .houses
            .into_iter()
            .map(|h| h.clone().fill_from_incubator())
            .collect_vec();
        let mut composed_world = World::new();
        for house in houses {
            house.unpack(composed_world.spawn_empty().id(), &mut composed_world);
        }
        rm(world).composed_world = composed_world;
        Ok(())
    }
    pub fn set_publish_nodes(mut node: impl Node, world: &mut World) {
        node.reassign_ids(&mut 0);
        let kind = node.kind();
        let nodes = node.to_tnodes();
        rm(world).new_node = Some((kind, nodes));
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
    pub fn pane_graph_core(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        let context = &Context::new_world(world);
        for house in core(world).houses_load(context) {
            house.view(ViewContext::full(), context, ui);
        }
        Ok(())
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
            kind.view_tnodes_mut(
                nodes,
                ViewContext {
                    mode: ViewMode::Graph,
                    ..default()
                },
                ui,
                world,
            );
            if "Publish".cstr_s(CstrStyle::Bold).button(ui).clicked() {
                cn().reducers
                    .incubator_push(nodes.clone(), d.new_node_link)
                    .unwrap();
                WindowPlugin::close_current(world);
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
        let Some(node) = cn().db.nodes_world().id().find(&id) else {
            return Err(format!("Failed to find node {kind}#{id}").into());
        };
        kind.view_tnodes(&[node].to_vec(), ViewContext::graph(), &default(), ui);
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
