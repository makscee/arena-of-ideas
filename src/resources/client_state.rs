use super::*;

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct ClientState {
    pub editor_teams: HashMap<Faction, PackedTeam>,
    pub editor: EditorResource,
}

static CLIENT_STATE: OnceCell<RwLock<ClientState>> = OnceCell::new();
const CLIENT_STATE_FILE: &str = "client_state.ron";

pub fn client_state() -> std::sync::RwLockReadGuard<'static, ClientState> {
    CLIENT_STATE.get_or_init(|| default()).read().unwrap()
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
