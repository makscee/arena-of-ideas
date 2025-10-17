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
                (NTeam::placeholder(), NTeam::placeholder())
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
        let mut state = world.resource_mut::<BattleEditorState>();
        let editor = TeamEditor::new();
        let (changed_team, _actions) = editor.edit(&state.left_team, ui);
        if let Some(new_team) = changed_team {
            dbg!(&new_team);
            state.left_team = new_team;
            save_changes_and_reload(&mut state);
        }
        if state.left_team.edit(ui).changed() {
            save_changes_and_reload(&mut state);
        }
    }
}

fn save_changes_and_reload(state: &mut Mut<'_, BattleEditorState>) {
    pd_mut(|pd| {
        pd.client_state
            .set_battle_test_teams(&state.left_team, &state.right_team)
    });

    let battle = Battle {
        id: 0,
        left: state.left_team.clone(),
        right: state.right_team.clone(),
    };
    state.simulation = BattleSimulation::new(battle).start();
}
