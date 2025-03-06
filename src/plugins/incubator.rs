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
    fn open_editor(kind: NodeKind, world: &mut World) {
        world.resource_mut::<IncubatorData>().edit_node =
            Some((NodeKind::Unit, kind.to_empty_strings()));
        DockPlugin::push(
            |dt| {
                if let Some(s) = dt.state.find_tab(&Tab::IncubatorNewNode) {
                    dt.state.set_active_tab(s);
                    dt.state.set_focused_node_and_surface((s.0, s.1));
                    return;
                }
                dt.state
                    .main_surface_mut()
                    .split_left(0.into(), 0.5, Tab::IncubatorNewNode.into());
            },
            world,
        );
    }

    pub fn tab_unit(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        ui.vertical(|ui| {
            let kind = NodeKind::Unit;
            let nodes = All::get_by_id(0, world)
                .unwrap()
                .incubator_load(world)?
                .units_load(world);
            for unit in nodes {
                unit.name.label(ui);
            }
            br(ui);
            if format!("New {kind}")
                .cstr_s(CstrStyle::Bold)
                .button(ui)
                .clicked()
            {
                Self::open_editor(kind, world);
            }
            Ok(())
        })
        .inner
    }
    pub fn tab_unit_stats(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        ui.vertical(|ui| {
            let kind = NodeKind::UnitStats;
            let nodes = All::get_by_id(0, world)
                .unwrap()
                .incubator_load(world)?
                .unit_stats_load(world);
            for node in nodes {
                format!(
                    "[b [yellow {}]] / [b [red {}]] (-{})",
                    node.pwr, node.hp, node.dmg
                )
                .label(ui);
            }
            br(ui);
            if format!("New {kind}")
                .cstr_s(CstrStyle::Bold)
                .button(ui)
                .clicked()
            {
                Self::open_editor(kind, world);
            }
            Ok(())
        })
        .inner
    }
    pub fn tab_new_node(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        world.resource_scope(|world, mut d: Mut<IncubatorData>| {
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
                    .incubator_push(kind.to_string(), datas.clone());
            }
        });
        Ok(())
    }
}
