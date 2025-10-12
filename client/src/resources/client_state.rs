use super::*;

#[derive(Deserialize, Serialize, Default, Debug, PartialEq, Clone)]
pub struct ClientState {
    pub last_logged_in: Option<(String, Identity)>,
    pub edit_anim: Option<Anim>,
    pub battle_test: (PackedNodes, PackedNodes),
    pub tile_states: HashMap<GameState, (Tree<Pane>, HashMap<TileId, String>)>,
}

impl PersistentData for ClientState {
    fn file_name() -> &'static str {
        "client_state"
    }
}

impl ClientState {
    pub fn get_battle_test_teams(&self) -> Option<(NTeam, NTeam)> {
        let left = &self.battle_test.0;
        let right = &self.battle_test.1;
        if left.root == 0 || right.root == 0 {
            return None;
        }
        let left = NTeam::unpack(left).unwrap();
        let right = NTeam::unpack(right).unwrap();
        Some((left, right))
    }
    pub fn set_battle_test_teams(&mut self, left: &NTeam, right: &NTeam) {
        self.battle_test.0 = left.pack();
        self.battle_test.1 = right.pack();
    }
}
