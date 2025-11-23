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
    pub was_playing: bool,
}

impl BattleEditorPlugin {
    pub fn pane_team_editor(is_left: bool, world: &mut World, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let saved_teams = pd().client_state.saved_teams.clone();

            for (name, left, right) in saved_teams.iter() {
                ui.menu_button(name, |ui| {
                    if ui.button("Load").clicked() {
                        let mut state = world.resource_mut::<BattleEditorState>();
                        let left_team = match NTeam::unpack(left) {
                            Ok(v) => v,
                            Err(e) => {
                                e.cstr().notify_error(world);
                                return;
                            }
                        };
                        let right_team = match NTeam::unpack(right) {
                            Ok(v) => v,
                            Err(e) => {
                                e.cstr().notify_error(world);
                                return;
                            }
                        };
                        state.left_team = left_team;
                        state.right_team = right_team;
                        BattleEditorPlugin::save_changes_and_reload(world);
                    }
                    if ui.button("Update").clicked() {
                        let state = world.resource_mut::<BattleEditorState>();
                        pd_mut(|pd| {
                            pd.client_state.save_team(
                                name.clone(),
                                &state.left_team,
                                &state.right_team,
                            );
                        });
                    }
                    if "[red Delete]".cstr().button(ui).clicked() {
                        pd_mut(|pd| {
                            pd.client_state.delete_team(name);
                        });
                    }
                });
            }

            if ui.button("+ Add").clicked() {
                let mut name = "name".to_owned();
                let confirmation = Confirmation::new("Save Team")
                    .accept_name("Save")
                    .cancel_name("Cancel")
                    .content(move |ui, world, result| {
                        Input::new("Team Name").ui_string(&mut name, ui);
                        ui.label("test");
                        if result == Some(true) {
                            if !name.is_empty() {
                                let state = world.resource::<BattleEditorState>();
                                let name = name.clone();
                                pd_mut(|pd| {
                                    pd.client_state.save_team(
                                        name,
                                        &state.left_team,
                                        &state.right_team,
                                    );
                                });
                            }
                        }
                    });
                confirmation.push(world);
            }
        });

        let mut changed_team = None;
        let needs_reload = false;
        if let Some(battle_data) = world.get_resource::<BattleData>() {
            let state = world.resource::<BattleEditorState>();
            let current_team = if is_left {
                &state.left_team
            } else {
                &state.right_team
            };

            if let Ok(result) =
                battle_data
                    .source
                    .exec_context_ref(|ctx| -> NodeResult<Option<NTeam>> {
                        let editor = TeamEditor::new()
                            .empty_slot_action(
                                "Add Placeholder Unit".to_string(),
                                |team, slot_index, _ctx, _ui| {
                                    let mut unit = NUnit::placeholder();
                                    if let Ok(houses) = team.houses.get_mut() {
                                        if let Some(house) = houses.first_mut() {
                                            if let Ok(slots) = team.slots.get_mut() {
                                                if let Some(slot) =
                                                    slots.iter_mut().find(|s| s.index == slot_index)
                                                {
                                                    if let RefMultiple::Ids(units) =
                                                        &mut house.units
                                                    {
                                                        units.push(unit.id);
                                                    }
                                                    unit.state_set(NUnitState::new(
                                                        next_id(),
                                                        unit.owner,
                                                        1,
                                                        0,
                                                    ))
                                                    .unwrap();
                                                    slot.unit = Owned::Loaded(unit);
                                                }
                                            }
                                        }
                                    }
                                },
                            )
                            .filled_slot_action(
                                "Inspect Unit".to_string(),
                                |team, unit_id, _slot_index, _ctx, ui| {
                                    ui.set_inspected_node_for_parent(team.id, unit_id);
                                },
                            );

                        Ok(editor.edit(current_team, ctx, ui))
                    })
            {
                changed_team = result;
            }
        }

        if let Some(new_team) = changed_team {
            let mut state = world.resource_mut::<BattleEditorState>();
            if is_left {
                state.left_team = new_team;
            } else {
                state.right_team = new_team;
            }
            BattleEditorPlugin::save_changes_and_reload(world);
        } else if needs_reload {
            BattleEditorPlugin::save_changes_and_reload(world);
        }
    }

    pub fn load_from_client_state(world: &mut World) {
        let (left_team, right_team) =
            if let Some((left, right)) = pd().client_state.get_battle_test_teams() {
                (left, right)
            } else {
                (NTeam::placeholder(), NTeam::placeholder())
            };
        world.insert_resource(BattleEditorState {
            left_team,
            right_team,
            was_playing: true,
        });
        Self::save_changes_and_reload(world);
    }

    pub fn pane_edit_graph(left: bool, ui: &mut Ui, world: &mut World) {
        if let Some(mut state) = world.get_resource_mut::<BattleEditorState>() {
            let needs_reload = if left {
                state.left_team.render_recursive_edit(ui)
            } else {
                state.right_team.render_recursive_edit(ui)
            };

            if needs_reload {
                BattleEditorPlugin::save_changes_and_reload(world);
            }
        } else {
            "[red [b BattleEditoState] not found]".cstr().label(ui);
        }
    }
    pub fn save_changes_and_reload(world: &mut World) {
        let was_playing = world
            .get_resource::<BattleData>()
            .map(|bd| bd.playing)
            .or_else(|| {
                world
                    .get_resource::<BattleEditorState>()
                    .map(|s| s.was_playing)
            })
            .unwrap_or(true);

        let state = world.resource_mut::<BattleEditorState>();
        pd_mut(|pd| {
            pd.client_state
                .set_battle_test_teams(&state.left_team, &state.right_team)
        });
        BattlePlugin::load_teams(0, state.left_team.clone(), state.right_team.clone(), world);

        if let Some(mut battle_data) = world.get_resource_mut::<BattleData>() {
            battle_data.playing = was_playing;
        }
        if let Some(mut editor_state) = world.get_resource_mut::<BattleEditorState>() {
            editor_state.was_playing = was_playing;
        }
    }
}
