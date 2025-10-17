use crate::prelude::*;

pub struct BattleEditorPlugin;

impl Plugin for BattleEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Editor), Self::load_from_client_state);
    }
}

#[derive(Resource)]
pub struct BattleEditorState {
    pub left_team: NTeam,
    pub right_team: NTeam,
    pub simulation: BattleSimulation,
}

impl BattleEditorPlugin {
    pub fn load_from_client_state(world: &mut World) {
        let (left_team, right_team) =
            if let Some((left, right)) = pd().client_state.get_battle_test_teams() {
                (left, right)
            } else {
                (NTeam::placeholder(next_id()), NTeam::placeholder(next_id()))
            };

        let battle = Battle {
            id: next_id(),
            left: left_team.clone(),
            right: right_team.clone(),
        };

        let simulation = BattleSimulation::new(battle).start();

        world.insert_resource(BattleEditorState {
            left_team,
            right_team,
            simulation,
        });
    }

    pub fn pane(world: &mut World, ui: &mut Ui) {
        let mut load_test_battle = false;
        let mut start_battle = false;

        ui.horizontal(|ui| {
            ui.heading("Battle Editor");

            if ui.button("Load Test Battle").clicked() {
                load_test_battle = true;
            }

            if ui.button("Start Battle").clicked() {
                start_battle = true;
            }
        });

        if load_test_battle {
            Self::load_from_client_state(world);
        }

        let mut state = world.resource_mut::<BattleEditorState>();

        if start_battle {
            let battle = Battle {
                id: next_id(),
                left: state.left_team.clone(),
                right: state.right_team.clone(),
            };
            state.simulation = BattleSimulation::new(battle).start();
        }

        ui.separator();

        egui::ScrollArea::both().show(ui, |ui| {
            ui.columns(2, |columns| {
                // Left team editor
                columns[0].group(|ui| {
                    ui.heading("Left Team");
                    ui.separator();

                    let editor = TeamEditor::new();
                    let (changed_team, _actions) = editor.edit(&state.left_team, ui);

                    if let Some(new_team) = changed_team {
                        state.left_team = new_team;
                        pd_mut(|pd| {
                            pd.client_state
                                .set_battle_test_teams(&state.left_team, &state.right_team)
                        });

                        let battle = Battle {
                            id: next_id(),
                            left: state.left_team.clone(),
                            right: state.right_team.clone(),
                        };
                        state.simulation = BattleSimulation::new(battle).start();
                    }
                });

                // Right team editor
                columns[1].group(|ui| {
                    ui.heading("Right Team");
                    ui.separator();

                    let editor = TeamEditor::new();
                    let (changed_team, _actions) = editor.edit(&state.right_team, ui);

                    if let Some(new_team) = changed_team {
                        state.right_team = new_team;
                        pd_mut(|pd| {
                            pd.client_state
                                .set_battle_test_teams(&state.left_team, &state.right_team)
                        });

                        let battle = Battle {
                            id: next_id(),
                            left: state.left_team.clone(),
                            right: state.right_team.clone(),
                        };
                        state.simulation = BattleSimulation::new(battle).start();
                    }
                });
            });
        });
    }
}
