use super::*;

pub struct BattlePlugin;

#[derive(Resource)]
struct BattleData {
    simulation: BattleSimulation,
    t: f32,
    playing: bool,
}

impl BattlePlugin {
    pub fn load_empty(world: &mut World) {
        world.insert_resource(BattleData {
            simulation: BattleSimulation::new(Battle {
                left: Team::default(),
                right: Team::default(),
            }),
            t: 0.0,
            playing: false,
        });
    }
    pub fn load(simulation: BattleSimulation, world: &mut World) {
        world.insert_resource(BattleData {
            simulation,
            t: 0.0,
            playing: false,
        });
    }
    pub fn add_panes() {
        TilePlugin::op(|tree| {
            if let Some(id) = tree.tiles.find_pane(&Pane::BattleView) {
                tree.tiles.remove(id);
            }
            if let Some(id) = tree.tiles.find_pane(&Pane::BattleControls) {
                tree.tiles.remove(id);
            }
            let view = tree.tiles.insert_pane(Pane::BattleView);
            let controls = tree.tiles.insert_pane(Pane::BattleControls);
            let id = tree.tiles.insert_vertical_tile([view, controls].into());
            tree.add_to_root(id).log();
        });
    }
    pub fn pane_view(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        let Some(mut data) = world.remove_resource::<BattleData>() else {
            "[red No battle loaded]".cstr().label(ui);
            return Ok(());
        };

        let t = data.t;
        data.simulation
            .show_at(t, ui, |slot, player_side, resp, ui| {
                resp.bar_menu(|ui| {
                    ui.menu_button("add unit", |ui| {
                        for unit in all(world).incubator_load(world)?.units_load(world) {
                            if unit.name.clone().button(ui).clicked() {
                                debug!("Unit {}", unit.name);
                                ui.close_menu();
                            }
                        }
                        Ok(())
                    })
                    .inner
                    .unwrap_or(Ok(()))
                    .log();
                });
            });
        world.insert_resource(data);
        Ok(())
    }
    pub fn pane_controls(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        let Some(mut data) = world.get_resource_mut::<BattleData>() else {
            "[red No battle loaded]".cstr().label(ui);
            return Ok(());
        };
        let BattleData {
            simulation,
            t,
            playing,
        } = data.as_mut();
        if simulation.duration > 0.0 {
            Slider::new("ts")
                .full_width()
                .ui(t, 0.0..=simulation.duration, ui);
        }
        ui.horizontal(|ui| {
            Checkbox::new(playing, "play").ui(ui);
            if "+1".cstr().button(ui).clicked() {
                simulation.run();
            }
            if "+10".cstr().button(ui).clicked() {
                for _ in 0..10 {
                    simulation.run();
                }
            }
            if "+100".cstr().button(ui).clicked() {
                for _ in 0..100 {
                    simulation.run();
                }
            }
        });
        if *playing {
            *t += gt().last_delta();
            *t = t.at_most(simulation.duration);
        }
        if *t >= simulation.duration && !simulation.ended() {
            simulation.run();
        }
        Ok(())
    }
}
