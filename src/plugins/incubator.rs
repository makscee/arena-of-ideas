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
    links_node: Option<(u64, NodeKind)>,
    link_types: Vec<NodeKind>,
    link_type_selected: NodeKind,
    callback_id: Option<IncubatorPushCallbackId>,
    edit_node: Option<(NodeKind, Vec<String>)>,
}

impl IncubatorPlugin {
    fn on_enter(world: &mut World) {
        let callback = cn().reducers.on_incubator_push(|e, kind, datas| {
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
        world.resource_mut::<IncubatorData>().callback_id = Some(callback);
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
            world.resource_mut::<IncubatorData>().edit_node = Some((kind, kind.to_empty_strings()));
            DockPlugin::set_active(Tab::IncubatorNewNode, world);
        }
    }
    pub fn tab_kind(kind: NodeKind, ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        let mut open_links: Option<(u64, NodeKind)> = None;
        ui.vertical(|ui| {
            match kind {
                NodeKind::Unit => {
                    for node in Self::incubator(world)?.units_load(world) {
                        node.name.label(ui);
                        if "links".cstr().button(ui).clicked() {
                            open_links = Some((node.id(), node.kind()));
                        }
                    }
                }
                NodeKind::UnitDescription => {
                    for node in Self::incubator(world)?.unit_descriptions_load(world) {
                        node.description.cstr_s(CstrStyle::Small).label(ui);
                    }
                }
                NodeKind::UnitStats => {
                    for node in Self::incubator(world)?.unit_stats_load(world) {
                        format!(
                            "[b [yellow {}]] / [b [red {}]] (-{})",
                            node.pwr, node.hp, node.dmg
                        )
                        .label(ui);
                    }
                }
                NodeKind::House => {
                    for node in Self::incubator(world)?.houses_load(world) {
                        node.name.cstr_s(CstrStyle::Small).label(ui);
                    }
                }
                NodeKind::Representation => {
                    for node in Self::incubator(world)?.representations_load(world) {
                        node.material.cstr_s(CstrStyle::Small).label(ui);
                    }
                }
                NodeKind::ActionAbility => {
                    for node in Self::incubator(world)?.action_abilities_load(world) {
                        node.name.cstr_s(CstrStyle::Small).label(ui);
                    }
                }
                NodeKind::ActionAbilityDescription => todo!(),
                NodeKind::AbilityEffect => todo!(),
                NodeKind::StatusAbility => todo!(),
                NodeKind::StatusAbilityDescription => todo!(),
                NodeKind::Reaction => todo!(),
                _ => unreachable!(),
            }
            Self::new_node_btn(kind, ui, world);
            if let Some((id, kind)) = open_links {
                let mut r = world.resource_mut::<IncubatorData>();
                r.links_node = Some((id, kind));
                r.link_types = NodeKind::get_incubator_links()
                    .get(&kind)
                    .unwrap()
                    .iter()
                    .sorted()
                    .copied()
                    .collect();
                r.link_type_selected = r.link_types[0];
                DockPlugin::set_active(Tab::IncubatorLinks, world);
            }
            Ok(())
        })
        .inner
    }
    pub fn tab_new_node(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        world.resource_scope(|_, mut d: Mut<IncubatorData>| {
            let Some((kind, datas)) = &mut d.edit_node else {
                "no editing node".cstr_c(VISIBLE_DARK).label(ui);
                return;
            };
            if Selector::new("Kind").ui_enum(kind, ui) {
                *datas = kind.to_empty_strings();
            }
            kind.show_strings_mut(datas, ui);
            if "Save".cstr_s(CstrStyle::Bold).button(ui).clicked() {
                cn().reducers
                    .incubator_push(kind.to_string(), datas.clone())
                    .unwrap();
            }
        });
        Ok(())
    }
    pub fn tab_links(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        let Some((id, kind)) = world.resource::<IncubatorData>().links_node else {
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
            if EnumSwitcher::new().show_iter(
                &mut selected,
                r.link_types.iter().copied(),
                ui,
            ) {
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
        .column_ui("node", |(_, node), _, ui, world| {
            let kind = node.kind.to_kind();
            let entity = world.get_id_link(node.id).unwrap();
            kind.show(entity, ui, world);
        })
        .column_cstr("data", |(_, node), _| node.data.cstr_s(CstrStyle::Small))
        .column_int("score", |(score, _)| *score)
        .ui(ui, world);

        Ok(())
    }
}
