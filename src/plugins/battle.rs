use super::*;

pub struct BattlePlugin;

#[derive(Resource)]
struct BattleData {
    battle: Battle,
    simulation: BattleSimulation,
    t: f32,
    playing: bool,
}

fn rm(world: &mut World) -> Mut<BattleData> {
    world.resource_mut::<BattleData>()
}

impl BattlePlugin {
    pub fn load_empty(world: &mut World) {
        let team = Team::default();
        let battle = Battle {
            left: team.clone(),
            right: team.clone(),
        };
        world.insert_resource(BattleData {
            simulation: BattleSimulation::new(battle.clone()),
            battle,
            t: 0.0,
            playing: false,
        });
    }
    fn reload_simulation(world: &mut World) {
        let mut data = rm(world);
        data.simulation = BattleSimulation::new(data.battle.clone()).start();
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
        data.simulation.show_at(t, ui);

        world.insert_resource(data);
        Ok(())
    }
    fn open_team_editor(left: bool, world: &mut World) {
        TeamEditorPlugin::load_team(
            if left {
                rm(world).battle.left.clone()
            } else {
                rm(world).battle.right.clone()
            },
            world,
        );
        TeamEditorPlugin::on_save_fn(
            move |team, world| {
                if left {
                    rm(world).battle.left = team;
                } else {
                    rm(world).battle.right = team;
                }
                Self::reload_simulation(world);
            },
            world,
        )
        .notify(world);
        TeamEditorPlugin::unit_add_from_core(world).notify(world);
    }
    pub fn pane_controls(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        let Some(mut data) = world.get_resource_mut::<BattleData>() else {
            "[red No battle loaded]".cstr().label(ui);
            return Ok(());
        };
        if "edit left team".cstr().button(ui).clicked() {
            op(|world| {
                Self::open_team_editor(true, world);
            });
        }
        if "edit right team".cstr().button(ui).clicked() {
            op(|world| {
                Self::open_team_editor(false, world);
            });
        }
        let BattleData {
            battle: _,
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
    fn add_incubator_unit(mut slot: usize, player_side: bool, unit: String, world: &mut World) {
        let mut data = rm(world);
        let team = if player_side {
            &mut data.battle.left
        } else {
            &mut data.battle.right
        };
        if slot > team.fusions.len() {
            slot = team.fusions.len();
            team.fusions.push(Fusion {
                slot: slot as i32,
                ..default()
            });
        } else {
            slot = slot - 1;
        }
        team.fusions[slot].units.push(unit);
        Self::reload_simulation(world);
    }
}
