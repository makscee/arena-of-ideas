use super::*;

#[derive(Deserialize, Serialize, Default, Debug, PartialEq, Clone)]
pub struct ClientState {
    pub last_logged_in: Option<(String, Identity)>,
    pub edit_anim: Option<Anim>,
    pub battle_test: (Vec<String>, Vec<String>),
    pub tile_states: HashMap<GameState, Tree<Pane>>,
}

impl PersistentData for ClientState {
    fn file_name() -> &'static str {
        "client_state"
    }
}

impl ClientState {
    pub fn get_battle_test_teams(&self) -> Option<(NTeam, NTeam)> {
        let left: Vec<TNode> = self.battle_test.0.iter().map(|n| n.into()).collect_vec();
        let right: Vec<TNode> = self.battle_test.1.iter().map(|n| n.into()).collect_vec();
        Some((
            NTeam::from_tnodes(left.get(0)?.id, &left)?,
            NTeam::from_tnodes(right.get(0)?.id, &right)?,
        ))
    }
    pub fn set_battle_test_teams(&mut self, left: &NTeam, right: &NTeam) {
        self.battle_test.0 = left.to_tnodes().into_iter().map(|n| n.into()).collect();
        self.battle_test.1 = right.to_tnodes().into_iter().map(|n| n.into()).collect();
    }
}
