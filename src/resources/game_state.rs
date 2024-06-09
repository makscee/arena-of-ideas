use super::*;

#[derive(
    Serialize, Deserialize, Clone, Copy, Eq, PartialEq, Debug, Hash, Default, States, Display,
)]
pub enum GameState {
    #[default]
    Loading,
    Loaded,
    Connect,
    CustomBattle,
    Battle,
    TestScenariosLoad,
    TestScenariosRun,
}

static TARGET_STATE: Mutex<GameState> = Mutex::new(GameState::Loaded);

lazy_static! {
    static ref STATE_PATHS: HashMap<GameState, Vec<GameState>> = {
        let mut m = HashMap::new();
        m.insert(
            GameState::CustomBattle,
            vec![
                GameState::Loaded,
                GameState::CustomBattle,
                GameState::Battle,
            ],
        );
        m.insert(
            GameState::TestScenariosRun,
            vec![GameState::Loaded, GameState::TestScenariosLoad],
        );
        m
    };
}

impl GameState {
    pub fn set_target(state: GameState) {
        *TARGET_STATE.lock().unwrap() = state;
    }
    fn get_target() -> GameState {
        *TARGET_STATE.lock().unwrap()
    }
}

pub struct GameStateGraphPlugin;

impl Plugin for GameStateGraphPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, Self::on_changed.run_if(state_changed::<GameState>));
    }
}

impl GameStateGraphPlugin {
    fn on_changed(state: Res<State<GameState>>, mut next_state: ResMut<NextState<GameState>>) {
        let state = state.get();
        info!("Enter state: {state}");
        if let Some(path) = STATE_PATHS.get(&GameState::get_target()) {
            if state.eq(path.last().unwrap()) {
                return;
            }
            for i in 0..path.len() {
                if state.eq(&path[i]) {
                    next_state.set(path[i + 1]);
                }
            }
        }
    }
}
