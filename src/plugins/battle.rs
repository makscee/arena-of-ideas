use bevy::app::FixedUpdate;

use super::*;

pub struct BattlePlugin;

impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Loaded), |world: &mut World| {
            if let Some((left, right)) = pd().client_state.get_battle_test_teams() {
                let battle = Battle { left, right };
                world.insert_resource(BattleData {
                    simulation: BattleSimulation::new(battle.clone()).start(),
                    battle,
                    t: 0.0,
                    playing: false,
                });
            } else {
                world.insert_resource(BattleData {
                    battle: Battle::default(),
                    simulation: BattleSimulation::new(default()).start(),
                    t: 0.0,
                    playing: false,
                });
            }
        })
        .init_resource::<ReloadData>()
        .add_systems(
            FixedUpdate,
            Self::reload.run_if(in_state(GameState::Editor)),
        );
    }
}

#[derive(Resource)]
struct BattleData {
    battle: Battle,
    simulation: BattleSimulation,
    t: f32,
    playing: bool,
}

#[derive(Resource, Default)]
struct ReloadData {
    reload_requested: bool,
    last_reload: f64,
}

fn rm(world: &mut World) -> Result<Mut<BattleData>, ExpressionError> {
    world
        .get_resource_mut::<BattleData>()
        .to_e("No battle loaded")
}

impl Default for BattleData {
    fn default() -> Self {
        let battle = Battle::default();
        Self {
            simulation: BattleSimulation::new(battle.clone()),
            battle,
            t: 0.0,
            playing: false,
        }
    }
}

impl BattleData {
    fn reload_simulation(&mut self) {
        self.simulation = BattleSimulation::new(self.battle.clone()).start();
    }
}

impl BattlePlugin {
    fn reload(mut data: ResMut<BattleData>, mut reload: ResMut<ReloadData>) {
        if reload.reload_requested && reload.last_reload + 0.1 < gt().elapsed() {
            reload.reload_requested = false;
            reload.last_reload = gt().elapsed();
            data.reload_simulation();
            data.battle.left.reassign_ids(&mut 0);
            data.battle.right.reassign_ids(&mut 0);
            pd_mut(|pd| {
                pd.client_state
                    .set_battle_test_teams(&data.battle.left, &data.battle.right);
            });
        }
    }
    pub fn edit_battle(f: impl FnOnce(&mut Battle), world: &mut World) {
        let mut r = world.get_resource_or_insert_with(|| BattleData::default());
        f(&mut r.battle);
    }
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
    pub fn pane_view(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        let mut data = world
            .remove_resource::<BattleData>()
            .to_e("No battle loaded")?;

        let t = data.t;
        data.simulation.show_at(t, ui);
        world.insert_resource(data);
        Ok(())
    }
    fn open_team_editor(left: bool, team: Team, world: &mut World) {
        TeamEditorPlugin::load_team(team, world);
        TeamEditorPlugin::on_save_fn(
            move |team, world| {
                match rm(world) {
                    Ok(mut data) => {
                        if left {
                            data.battle.left = team;
                        } else {
                            data.battle.right = team;
                        }
                        data.reload_simulation();
                    }
                    Err(e) => format!("Failed to save: {e}").notify(world),
                }
                TilePlugin::close_match(|p| matches!(p, Pane::Team(..)));
            },
            world,
        )
        .notify(world);
        TeamEditorPlugin::unit_add_from_core(world).notify(world);
        TeamEditorPlugin::add_panes();
    }
    pub fn pane_controls(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        let mut data = world
            .remove_resource::<BattleData>()
            .to_e("No battle loaded")?;
        if "edit left team".cstr().button(ui).clicked() {
            Self::open_team_editor(true, data.battle.left.clone(), world);
        }
        if "edit right team".cstr().button(ui).clicked() {
            Self::open_team_editor(false, data.battle.right.clone(), world);
        }
        let BattleData {
            battle: _,
            simulation,
            t,
            playing,
        } = &mut data;
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
        world.insert_resource(data);
        Ok(())
    }
    pub fn pane_edit(left: bool, ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        let mut data = rm(world)?;
        let changed = if left {
            &mut data.battle.left
        } else {
            &mut data.battle.right
        }
        .graph_view_mut(Rect::ZERO, ui);
        if changed {
            world.resource_mut::<ReloadData>().reload_requested = true;
        }
        Ok(())
    }
}
