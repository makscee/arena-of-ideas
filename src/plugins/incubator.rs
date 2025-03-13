use super::*;

pub struct IncubatorPlugin;

impl Plugin for IncubatorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<IncubatorData>()
            .add_systems(OnEnter(GameState::Incubator), Self::on_enter)
            .add_systems(OnExit(GameState::Incubator), Self::on_exit);
    }
}

#[derive(Resource, Default)]
struct IncubatorData {
    table_kind: NodeKind,
    inspect_node: Option<(u64, NodeKind)>,
    link_types: Vec<NodeKind>,
    link_type_selected: NodeKind,
    callback_id: Option<IncubatorPushCallbackId>,
    edit_node: Option<(NodeKind, Vec<String>)>,
}
fn rm(world: &mut World) -> Mut<IncubatorData> {
    world.resource_mut::<IncubatorData>()
}

impl IncubatorPlugin {
    fn on_enter(world: &mut World) {
        let callback = cn().reducers.on_incubator_push(|e, kind, _| {
            if !e.check_identity() {
                return;
            }
            let kind = NodeKind::from_str(kind).unwrap();
            e.event.on_success_error(
                move || {
                    format!("New {kind} added").notify_op();
                },
                move || {
                    format!("Failed to add new {kind}").notify_op();
                },
            );
        });
        let mut r = rm(world);
        r.callback_id = Some(callback);
        r.table_kind = NodeKind::House;
    }
    fn on_exit(world: &mut World) {
        world.resource_scope(|_, mut d: Mut<IncubatorData>| {
            if let Some(callback) = d.callback_id.take() {
                cn().reducers.remove_on_incubator_push(callback);
            } else {
                error!("New incubator node reducer callback not set");
            }
        });
    }
    fn incubator<'a>(world: &'a World) -> Result<&'a Incubator, ExpressionError> {
        All::get_by_id(0, world).unwrap().incubator_load(world)
    }
    fn new_node_btn(kind: NodeKind, ui: &mut Ui, world: &mut World) {
        br(ui);
        if format!("New {kind}")
            .cstr_s(CstrStyle::Bold)
            .button(ui)
            .clicked()
        {
            todo!();
            // world.resource_mut::<IncubatorData>().edit_node = Some((kind, kind.to_empty_strings()));
            DockPlugin::set_active(Tab::IncubatorNewNode, world);
        }
    }
    pub fn tab_nodes(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        let mut data = rm(world);
        let kind = data.table_kind;
        NodeKind::House.show_graph(CYAN, &mut data, ui);
        ui.vertical(|ui| {
            Table::new(kind.to_string(), |world| {
                let incubator = Self::incubator(world).unwrap().id();
                let kind = kind.to_string();
                cn().db
                    .nodes_world()
                    .iter()
                    .filter(|n| n.parent == incubator && n.kind == kind)
                    .map(|n| n.id)
                    .collect_vec()
            })
            .add_node_view_columns(kind, |d| *d)
            .add_incubator_columns(kind, |d| *d)
            .ui(ui, world);
            Self::new_node_btn(kind, ui, world);
            Ok(())
        })
        .inner
    }
    pub fn tab_new_node(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        world.resource_scope(|_, mut d: Mut<IncubatorData>| {
            let (kind, datas) = if let Some((kind, datas)) = &mut d.edit_node {
                (kind, datas)
            } else {
                d.edit_node = Some((NodeKind::Unit, default()));
                let node = d.edit_node.as_mut().unwrap();
                (&mut node.0, &mut node.1)
            };
            if Selector::new("Kind").ui_iter(kind, Incubator::children_kinds().iter(), ui) {
                // *datas = kind.to_empty_strings();
            }
            // kind.show_tnodes_mut(datas, ui);
            if "Save".cstr_s(CstrStyle::Bold).button(ui).clicked() {
                // cn().reducers
                //     .incubator_push(kind.to_string(), datas.clone())
                //     .unwrap();
            }
        });
        Ok(())
    }
    pub fn tab_links(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        let Some((id, kind)) = world.resource::<IncubatorData>().inspect_node else {
            "Select node to view links".cstr_c(VISIBLE_DARK).label(ui);
            return Ok(());
        };
        format!("{kind} Links")
            .cstr_s(CstrStyle::Heading2)
            .label(ui);
        kind.show(
            world
                .get_id_link(id)
                .to_e_fn(|| format!("Failed to find node#{id}"))?,
            ui,
            world,
        );

        let mut r = world.resource_mut::<IncubatorData>();
        let mut selected = r.link_type_selected;
        ui.horizontal(|ui| {
            "show type".cstr_c(VISIBLE_DARK).label(ui);
            if EnumSwitcher::new().show_iter(&mut selected, r.link_types.iter().copied(), ui) {
                r.link_type_selected = selected;
            }
        });
        Table::new(format!("{selected} links"), move |world| {
            let incubator_id = Self::incubator(world).unwrap().id();
            let kind = selected.to_string();
            cn().db
                .nodes_world()
                .iter()
                .filter_map(|r| {
                    if r.parent == incubator_id && r.kind == kind {
                        Some((
                            cn().db
                                .incubator_links()
                                .iter()
                                .find_map(|l| {
                                    if l.from == id && l.to_kind == kind && l.to == r.id {
                                        Some(l.score as i32)
                                    } else {
                                        None
                                    }
                                })
                                .unwrap_or_default(),
                            r,
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
            "-",
            move |(_, node), _, _| {
                cn().reducers.incubator_vote(id, node.id, -1).unwrap();
            },
            move |(_, node), _, btn| {
                if let Some(vote) =
                    cn().db
                        .incubator_votes()
                        .key()
                        .find(&vote_key(player_id(), id, node.id))
                {
                    btn.active(vote.vote == -1)
                } else {
                    btn
                }
            },
        )
        .column_btn_mod_dyn(
            "+",
            move |(_, node), _, _| {
                cn().reducers.incubator_vote(id, node.id, 1).unwrap();
            },
            move |(_, node), _, btn| {
                if let Some(vote) =
                    cn().db
                        .incubator_votes()
                        .key()
                        .find(&vote_key(player_id(), id, node.id))
                {
                    btn.active(vote.vote == 1)
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
        self = self.column_btn_dyn("clone", move |d, _, world| {
            let id = f(d);
            let node = cn().db.nodes_world().id().find(&id).unwrap();
            let kind = node.kind();
            world.resource_mut::<IncubatorData>().edit_node =
                Some((kind, [format!("0 _ {}", node.data)].into()));
            DockPlugin::set_active(Tab::IncubatorNewNode, world);
        });
        if NodeKind::get_incubator_links()
            .get(&kind)
            .is_some_and(|l| !l.is_empty())
        {
            self.column_btn_dyn("links", move |d, _, world| {
                let mut r = world.resource_mut::<IncubatorData>();
                r.inspect_node = Some((f(d), kind));
                r.link_types = NodeKind::get_incubator_links()
                    .get(&kind)
                    .unwrap()
                    .iter()
                    .sorted()
                    .copied()
                    .collect();
                r.link_type_selected = r.link_types[0];
                DockPlugin::set_active(Tab::IncubatorLinks, world);
            })
        } else {
            self
        }
    }
}

fn vote_key(player: u64, from: u64, to: u64) -> String {
    format!("{player}_{from}_{to}")
}

trait NodeKindGraph {
    fn show_graph(self, color: Color32, data: &mut IncubatorData, ui: &mut Ui);
}

impl NodeKindGraph for NodeKind {
    fn show_graph(self, mut color: Color32, data: &mut IncubatorData, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if data.inspect_node.is_some_and(|(_, k)| k == self) {
                color = PURPLE;
            }
            if self
                .cstr_cs(color, CstrStyle::Small)
                .as_button()
                .active(data.table_kind == self)
                .ui(ui)
                .clicked()
            {
                data.table_kind = self;
            }
            ui.vertical(|ui| {
                for c in self.component_kinds() {
                    c.show_graph(VISIBLE_LIGHT, data, ui);
                }
                for c in self.children_kinds() {
                    c.show_graph(RED, data, ui);
                }
            });
        });
    }
}
