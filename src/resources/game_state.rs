use super::*;

#[derive(
    Serialize, Deserialize, Clone, Copy, Eq, PartialEq, Debug, Hash, Default, States, Display,
)]
pub enum GameState {
    #[default]
    Loading,
    Loaded,
    Title,
    Connect,
    CustomBattle,
    Battle,
    TestScenariosLoad,
    TestScenariosRun,
    ServerSync,
}

static TARGET_STATE: Mutex<GameState> = Mutex::new(GameState::Loaded);

lazy_static! {
    static ref STATE_PATHS: HashMap<GameState, Vec<GameState>> = {
        let mut m = HashMap::new();
        m.insert(GameState::Title, vec![GameState::Loaded, GameState::Title]);
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
        m.insert(
            GameState::ServerSync,
            vec![GameState::Loaded, GameState::Connect, GameState::ServerSync],
        );
        m
    };
}

impl GameState {
    pub fn set_target(self) {
        *TARGET_STATE.lock().unwrap() = self;
    }
    pub fn get_target() -> GameState {
        *TARGET_STATE.lock().unwrap()
    }
    pub fn change(self, world: &mut World) {
        self.set_target();
        world
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Loaded);
    }
}

pub struct GameStatePlugin;

impl Plugin for GameStatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, Self::on_changed.run_if(state_changed::<GameState>))
            .add_systems(Update, Self::update);
    }
}

impl GameStatePlugin {
    pub fn on_changed(state: Res<State<GameState>>, mut next_state: ResMut<NextState<GameState>>) {
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
    fn update(time: Res<Time>) {
        let mut timer = GameTimer::get();
        timer.update(time.delta_seconds());
    }
}
