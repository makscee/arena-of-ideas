use bevy::app::FixedUpdate;

use super::*;

pub struct BattlePlugin;

impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Editor), Self::load_from_client_state)
            .init_resource::<ReloadData>()
            .add_systems(
                FixedUpdate,
                Self::reload.run_if(in_state(GameState::Editor)),
            );
    }
}

#[derive(Resource)]
struct BattleData {
    teams_world: World,
    team_left: Entity,
    team_right: Entity,
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

impl BattleData {
    fn load(battle: Battle) -> Self {
        let mut teams_world = World::new();
        let team_left = teams_world.spawn_empty().id();
        let team_right = teams_world.spawn_empty().id();
        battle.left.clone().unpack(team_left, &mut teams_world);
        battle.right.clone().unpack(team_right, &mut teams_world);
        let simulation = BattleSimulation::new(battle.clone()).start();
        Self {
            teams_world,
            team_left,
            team_right,
            battle,
            simulation,
            t: 0.0,
            playing: false,
        }
    }
}

impl BattlePlugin {
    pub fn load_from_client_state(world: &mut World) {
        let bd = if let Some((left, right)) = pd().client_state.get_battle_test_teams() {
            let battle = Battle { left, right };
            BattleData::load(battle)
        } else {
            BattleData::load(default())
        };
        world.insert_resource(bd);
    }
    fn reload(mut data: ResMut<BattleData>, mut reload: ResMut<ReloadData>) {
        if reload.reload_requested && reload.last_reload + 0.1 < gt().elapsed() {
            reload.reload_requested = false;
            reload.last_reload = gt().elapsed();
            *data = BattleData::load(data.battle.clone());
            pd_mut(|pd| {
                pd.client_state
                    .set_battle_test_teams(&data.battle.left, &data.battle.right);
            });
        }
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
    pub fn pane_controls(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        let mut data = world
            .remove_resource::<BattleData>()
            .to_e("No battle loaded")?;
        let BattleData {
            teams_world: _,
            battle: _,
            simulation,
            t,
            playing,
            team_left: _,
            team_right: _,
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
    pub fn pane_edit_graph(left: bool, ui: &mut Ui, world: &mut World) {
        world.resource_scope(|world, mut data: Mut<BattleData>| {
            let team = if left {
                &mut data.battle.left
            } else {
                &mut data.battle.right
            };
            let mut changed = false;
            changed |= team
                .view_mut(ViewContext::new(ui), &world.into(), ui)
                .changed;
            if changed {
                world.resource_mut::<ReloadData>().reload_requested = true;
            }
        });
    }
    pub fn pane_edit_slots(left: bool, ui: &mut Ui, world: &mut World) {
        world.resource_scope(|world, mut data: Mut<BattleData>| {
            let BattleData {
                teams_world,
                team_left,
                team_right,
                battle,
                simulation: _,
                t: _,
                playing: _,
            } = data.as_mut();
            let mut changed = false;
            let team_entity = if left { *team_left } else { *team_right };
            let mut edited = None;
            let mut add_fusion = false;
            let context = &teams_world.into();
            Fusion::slots_editor(
                team_entity,
                context,
                ui,
                |ui| {
                    ui.vertical_centered_justified(|ui| {
                        if "add fusion".cstr().button(ui).clicked() {
                            add_fusion = true;
                        }
                    });
                },
                |fusion| {
                    edited = Some(fusion);
                },
            )
            .ui(ui);
            if add_fusion {
                let team = Team::get(team_entity, context).unwrap();
                let slot = team.fusions_load(context).len() as i32;
                let id = next_id();
                Fusion {
                    id,
                    parent: team.id(),
                    entity: None,
                    units: Vec::new(),
                    behavior: Vec::new(),
                    slot,
                    stats: None,
                }
                .unpack(
                    teams_world.spawn_empty().set_parent(team_entity).id(),
                    teams_world,
                );
            }
            if let Some(fusion) = edited {
                teams_world.entity_mut(fusion.entity()).insert(fusion);
                changed = true;
            }

            if changed {
                world.resource_mut::<ReloadData>().reload_requested = true;
                let updated_team = Team::pack(team_entity, &teams_world.into()).unwrap();
                let team = if left {
                    &mut battle.left
                } else {
                    &mut battle.right
                };
                *team = updated_team;
            }
        });
    }
}
