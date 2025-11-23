use super::*;

#[derive(Deserialize, Serialize, Default, Debug, PartialEq, Clone)]
pub struct ClientState {
    pub last_logged_in: Option<(String, Identity)>,
    pub edit_anim: Option<Anim>,
    pub battle_test: (PackedNodes, PackedNodes),
    pub tile_states: HashMap<GameState, (Tree<Pane>, HashMap<TileId, String>)>,
    pub saved_teams: Vec<(String, PackedNodes, PackedNodes)>,
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

    pub fn save_team(&mut self, name: String, left: &NTeam, right: &NTeam) {
        if let Some((_, l, r)) = self.saved_teams.iter_mut().find(|(n, _, _)| name.eq(n)) {
            *l = left.pack();
            *r = right.pack();
            return;
        } else {
            let packed = (name, left.pack(), right.pack());
            self.saved_teams.push(packed);
        }
    }

    pub fn delete_team(&mut self, name: &str) {
        self.saved_teams.retain(|(n, _, _)| n != name);
    }

    pub fn get_team(&self, name: &str) -> Option<(NTeam, NTeam)> {
        self.saved_teams
            .iter()
            .find(|(n, _, _)| n == name)
            .and_then(|(_, left, right)| {
                let l = NTeam::unpack(left).ok()?;
                let r = NTeam::unpack(right).ok()?;
                Some((l, r))
            })
    }
}
