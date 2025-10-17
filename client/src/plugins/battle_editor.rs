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
        });
        Self::save_changes_and_reload(world);
    }

    pub fn pane(world: &mut World, ui: &mut Ui) {
        let mut state = world.resource_mut::<BattleEditorState>();
        let editor = TeamEditor::new();
        let (changed_team, _actions) = editor.edit(&state.left_team, ui);
        if let Some(new_team) = changed_team {
            dbg!(&new_team);
            state.left_team = new_team;
            Self::save_changes_and_reload(world);
        } else if state.left_team.edit(ui).changed() {
            Self::save_changes_and_reload(world);
        }
    }
    fn save_changes_and_reload(world: &mut World) {
        let state = world.resource_mut::<BattleEditorState>();
        pd_mut(|pd| {
            pd.client_state
                .set_battle_test_teams(&state.left_team, &state.right_team)
        });
        BattlePlugin::load_teams(0, state.left_team.clone(), state.right_team.clone(), world);
    }
}
