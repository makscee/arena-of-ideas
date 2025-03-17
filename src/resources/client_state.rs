use spacetimedb_sats::serde::SerdeWrapper;

use super::*;

#[derive(Deserialize, Serialize, Clone, Default)]
pub struct ClientState {
    pub last_logged_in: Option<(String, Identity)>,
    pub edit_anim: Option<Anim>,
    pub battle_test_teams: (Vec<String>, Vec<String>),
    pub tile_states: HashMap<GameState, Tree<Pane>>,
}

static CLIENT_STATE: OnceCell<RwLock<ClientState>> = OnceCell::new();
const CLIENT_STATE_FILE: &str = "client_state.ron";

pub fn client_state() -> std::sync::RwLockReadGuard<'static, ClientState> {
    CLIENT_STATE.get_or_init(|| default()).read().unwrap()
}
impl ClientState {
    pub fn get_battle_test_teams(&self) -> (Vec<TNode>, Vec<TNode>) {
        (
            self.battle_test_teams
                .0
                .iter()
                .map(|n| ron::from_str::<SerdeWrapper<TNode>>(n).unwrap().0)
                .collect_vec(),
            self.battle_test_teams
                .1
                .iter()
                .map(|n| ron::from_str::<SerdeWrapper<TNode>>(n).unwrap().0)
                .collect_vec(),
        )
    }
}
fn path() -> PathBuf {
    let mut path = home_dir_path();
    path.push(CLIENT_STATE_FILE);
    path
}
pub fn load_client_state() {
    let cs = if let Some(cs) = std::fs::read_to_string(&path())
        .ok()
        .and_then(|d| ron::from_str::<ClientState>(d.leak()).ok())
    {
        cs
    } else {
        ClientState::default().save_to_file()
    };
    cs.save_to_cache();
}

impl ClientState {
    pub fn save(self) {
        self.save_to_file().save_to_cache();
    }
    pub fn save_to_cache(self) {
        *CLIENT_STATE.get_or_init(|| default()).write().unwrap() = self;
    }
    pub fn save_to_file(self) -> Self {
        match std::fs::write(
            path(),
            to_string_pretty(&self, PrettyConfig::new())
                .expect("Failed to serialize default client settings"),
        ) {
            Ok(_) => {
                info!("State store successful")
            }
            Err(e) => {
                error!("Store error: {e}")
            }
        }
        self
    }
}
