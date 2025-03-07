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
        world.resource_scope(|world, mut d: Mut<IncubatorData>| {
            if let Some(callback) = d.callback_id.take() {
                cn().reducers.remove_on_incubator_push(callback);
            } else {
                format!("New incubator node reducer callback not set").notify_error(world);
            }
        });
    }
    fn open_links(id: u64, kind: NodeKind, world: &mut World) {
        world.resource_mut::<IncubatorData>().links_node = Some((id, kind));
        GameState::IncubatorLinks.set_next(world);
    }
    fn open_editor(kind: NodeKind, world: &mut World) {
        world.resource_mut::<IncubatorData>().edit_node = Some((kind, kind.to_empty_strings()));
        DockPlugin::push(
            |dt| {
                if let Some(s) = dt.state.find_tab(&Tab::IncubatorNewNode) {
                    dt.state.set_active_tab(s);
                    dt.state.set_focused_node_and_surface((s.0, s.1));
                    return;
                }
                dt.state.main_surface_mut().split_below(
                    0.into(),
                    0.5,
                    Tab::IncubatorNewNode.into(),
                );
            },
            world,
        );
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
            Self::open_editor(kind, world)
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
                Self::open_links(id, kind, world);
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
        let (id, kind) = world.resource::<IncubatorData>().links_node.unwrap();
        const LINKS: LazyCell<HashMap<NodeKind, HashSet<NodeKind>>> =
            LazyCell::new(|| NodeKind::get_incubator_links());
        let links = LINKS.get(&kind).cloned().unwrap();
        ui.columns(links.len() + 1, |ui| {
            kind.cstr().label(&mut ui[0]);
            for (i, kind) in links.iter().enumerate() {
                kind.cstr().label(&mut ui[i + 1]);
            }
        });

        Ok(())
    }
}
