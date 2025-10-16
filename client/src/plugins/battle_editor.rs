use crate::prelude::*;

pub struct BattleEditorPlugin;

impl Plugin for BattleEditorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BattleEditorState>();
    }
}

#[derive(Resource)]
pub struct BattleEditorState {
    pub left_team: NTeam,
    pub right_team: NTeam,
    pub simulation: Option<BattleSimulation>,
}

impl Default for BattleEditorState {
    fn default() -> Self {
        let mut left_team = NTeam::new(next_id());
        let mut right_team = NTeam::new(next_id());

        let unit1 = NUnit::new(next_id(), "Unit 1".to_string()).add_components(
            NUnitDescription::placeholder(next_id()),
            NUnitStats::new(next_id(), 5, 10),
            NUnitState::new(next_id(), 1),
        );

        let unit2 = NUnit::new(next_id(), "Unit 2".to_string()).add_components(
            NUnitDescription::placeholder(next_id()),
            NUnitStats::new(next_id(), 7, 8),
            NUnitState::new(next_id(), 1),
        );

        let house = NHouse::new(next_id(), "Test House".to_string()).add_components(
            NHouseColor::placeholder(next_id()),
            NAbilityMagic::placeholder(next_id()),
            NStatusMagic::placeholder(next_id()),
            vec![unit1.clone(), unit2.clone()],
        );

        left_team.houses.get_mut().unwrap().push(house.clone());
        right_team.houses.get_mut().unwrap().push(house);

        let left_fusion =
            NFusion::new(next_id(), left_team.id, 0, 5, 10, 0, 1).add_components(vec![
                NFusionSlot::new(
                    next_id(),
                    0,
                    UnitActionRange {
                        trigger: 0,
                        start: 0,
                        length: 1,
                    },
                )
                .add_components(unit1.clone()),
            ]);

        let right_fusion =
            NFusion::new(next_id(), right_team.id, 0, 7, 8, 0, 1).add_components(vec![
                NFusionSlot::new(
                    next_id(),
                    0,
                    UnitActionRange {
                        trigger: 0,
                        start: 0,
                        length: 1,
                    },
                )
                .add_components(unit2.clone()),
            ]);

        left_team.fusions.get_mut().unwrap().push(left_fusion);
        right_team.fusions.get_mut().unwrap().push(right_fusion);

        Self {
            left_team,
            right_team,
            simulation: None,
        }
    }
}

impl BattleEditorPlugin {
    pub fn init(commands: &mut Commands) {
        commands.insert_resource(BattleEditorState::default());
    }

    pub fn pane(world: &mut World, ui: &mut Ui) {
        let mut state = world.resource_mut::<BattleEditorState>();

        ui.horizontal(|ui| {
            ui.heading("Battle Editor");

            if ui.button("Load Test Battle").clicked() {
                *state = BattleEditorState::default();
            }

            if ui.button("Start Battle").clicked() {
                let battle = Battle {
                    id: next_id(),
                    left: state.left_team.clone(),
                    right: state.right_team.clone(),
                };

                state.simulation = Some(BattleSimulation::new(battle).start());
            }
        });

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

                        // Reload simulation if it exists
                        if state.simulation.is_some() {
                            let battle = Battle {
                                id: next_id(),
                                left: state.left_team.clone(),
                                right: state.right_team.clone(),
                            };
                            state.simulation = Some(BattleSimulation::new(battle).start());
                        }
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

                        // Reload simulation if it exists
                        if state.simulation.is_some() {
                            let battle = Battle {
                                id: next_id(),
                                left: state.left_team.clone(),
                                right: state.right_team.clone(),
                            };
                            state.simulation = Some(BattleSimulation::new(battle).start());
                        }
                    }
                });
            });

            ui.separator();

            // Battle pane
            let has_simulation = state.simulation.is_some();
            if has_simulation {
                ui.group(|ui| {
                    ui.heading("Battle Simulation");
                    ui.separator();

                    let mut should_reset = false;

                    ui.horizontal(|ui| {
                        if ui.button("Step").clicked() {
                            if let Some(simulation) = &mut state.simulation {
                                simulation.run();
                            }
                        }

                        if ui.button("Run to End").clicked() {
                            if let Some(simulation) = &mut state.simulation {
                                while !simulation.ended() {
                                    simulation.run();
                                }
                            }
                        }

                        if ui.button("Reset").clicked() {
                            should_reset = true;
                        }

                        if let Some(simulation) = &state.simulation {
                            ui.label(format!("Time: {:.1}", simulation.t));
                        }
                    });

                    if should_reset {
                        let battle = Battle {
                            id: next_id(),
                            left: state.left_team.clone(),
                            right: state.right_team.clone(),
                        };
                        state.simulation = Some(BattleSimulation::new(battle).start());
                    }

                    ui.separator();

                    if let Some(simulation) = &mut state.simulation {
                        BattleCamera::show(simulation, simulation.t, ui);
                    }
                });
            }
        });
    }
}
