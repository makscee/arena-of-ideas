use crate::prelude::*;

pub struct BattleEditorPlugin;

pub struct TeamEditorPlugin;

impl Plugin for BattleEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Editor), Self::load_from_client_state)
            .add_systems(
                Update,
                Self::handle_space_input.run_if(in_state(GameState::Editor)),
            );
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
            let t = battle_data.t;
            let state = world.resource::<BattleEditorState>();
            let current_team = if is_left {
                &state.left_team
            } else {
                &state.right_team
            };

            if let Ok(result) = battle_data.simulation.with_context(
                t,
                |ctx| -> NodeResult<(Option<NTeam>, Vec<TeamAction>)> {
                    let editor = TeamEditor::new();
                    Ok(editor.edit(current_team, ctx, ui))
                },
            ) {
                changed_team = result.0;
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

    pub fn handle_space_input(
        input: Res<ButtonInput<KeyCode>>,
        state: Res<State<GameState>>,
        battle_data: Option<ResMut<BattleData>>,
        editor_state: Option<ResMut<BattleEditorState>>,
    ) {
        if state.get() != &GameState::Editor {
            return;
        }

        if input.just_pressed(KeyCode::Space) {
            if let (Some(mut battle_data), Some(mut editor_state)) = (battle_data, editor_state) {
                battle_data.playing = !battle_data.playing;
                editor_state.was_playing = battle_data.playing;
            }
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

        let state = world.resource::<BattleEditorState>();
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
