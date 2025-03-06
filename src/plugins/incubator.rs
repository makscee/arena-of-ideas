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
    callback_id: Option<IncubatorNewNodeCallbackId>,
    new_unit: Unit,
}

impl IncubatorPlugin {
    fn on_enter(world: &mut World) {
        let callback = cn().reducers.on_incubator_new_node(|e, kind, data| {
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
                cn().reducers.remove_on_incubator_new_node(callback);
            } else {
                format!("New incubator node reducer callback not set").notify_error(world);
            }
        });
    }
    pub fn tab_units(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        ui.vertical(|ui| {
            let units = All::get_by_id(0, world)
                .unwrap()
                .incubator_load(world)?
                .units_load(world);
            for unit in units {
                unit.name.label(ui);
            }
            br(ui);
            if "New Unit".cstr_s(CstrStyle::Bold).button(ui).clicked() {
                DockPlugin::push(
                    |dt| {
                        if let Some(s) = dt.state.find_tab(&Tab::IncubatorNewUnit) {
                            dt.state.set_active_tab(s);
                            dt.state.set_focused_node_and_surface((s.0, s.1));
                            return;
                        }
                        let Some((_, n, _)) = dt.state.find_tab(&Tab::IncubatorUnits) else {
                            error!("Can't find units tab");
                            return;
                        };
                        dt.state.main_surface_mut().split_below(
                            n,
                            0.5,
                            Tab::IncubatorNewUnit.into(),
                        );
                    },
                    world,
                );
            }
            Ok(())
        })
        .inner
    }
    pub fn tab_new_unit(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        world.resource_scope(|_, mut d: Mut<IncubatorData>| {
            let unit = &mut d.new_unit;
            Input::new("unit name").ui_string(&mut unit.name, ui);
            if "Save".cstr_s(CstrStyle::Bold).button(ui).clicked() {
                cn().reducers
                    .incubator_new_node(unit.kind().to_string(), unit.get_data())
                    .unwrap();
            }
        });
        Ok(())
    }
}
