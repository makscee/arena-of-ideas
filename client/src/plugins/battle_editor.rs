use crate::prelude::*;

pub struct BattleEditorPlugin;

pub struct TeamEditorPlugin;

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

impl TeamEditorPlugin {
    pub fn pane(is_left: bool, world: &mut World, ui: &mut Ui) {
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
                                Box::new(
                                    |team: &mut NTeam,
                                     fusion_id: u64,
                                     slot_index: i32,
                                     _ctx: &ClientContext,
                                     _ui: &mut Ui| {
                                        let unit = NUnit::placeholder();
                                        let unit_id = unit.id;
                                        team.houses
                                            .get_mut()
                                            .unwrap()
                                            .first_mut()
                                            .unwrap()
                                            .units_push(unit)
                                            .unwrap();
                                        let slot =
                                            team.fusion_slot_mut(fusion_id, slot_index).unwrap();
                                        slot.unit = Ref::new_id(unit_id);
                                        slot.set_dirty(true);
                                    },
                                ),
                            )
                            .filled_slot_action(
                                "Inspect Unit".to_string(),
                                Box::new(
                                    |team: &mut NTeam,
                                     _fusion_id: u64,
                                     unit_id: u64,
                                     _slot_index: i32,
                                     _ctx: &ClientContext,
                                     ui: &mut Ui| {
                                        // Set inspected node using InspectedNodeExt
                                        ui.set_inspected_node_for_parent(team.id, unit_id);
                                    },
                                ),
                            );

                        Ok(editor.edit(current_team, ctx, ui))
                    })
            {
                changed_team = result;
            }
        }

        if let Some(mut new_team) = changed_team {
            // Fix team integrity after any changes
            new_team.fix_integrity().notify_error_op();

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
}

impl BattleEditorPlugin {
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

        let mut state = world.resource_mut::<BattleEditorState>();
        state.left_team.fix_integrity().unwrap();
        state.right_team.fix_integrity().unwrap();
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
