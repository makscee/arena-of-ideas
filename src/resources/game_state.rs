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
    Login,
    CustomBattle,
    Battle,
    Shop,
    TestScenariosLoad,
    TestScenariosRun,
    ServerSync,
    Profile,
}

static TARGET_STATE: Mutex<GameState> = Mutex::new(GameState::Loaded);
static STATE_READY: Mutex<bool> = Mutex::new(true);

lazy_static! {
    static ref STATE_PATHS: HashMap<GameState, Vec<GameState>> = {
        let mut m = HashMap::new();
        m.insert(
            GameState::Title,
            vec![
                GameState::Loaded,
                GameState::Connect,
                GameState::Login,
                GameState::Title,
            ],
        );
        m.insert(
            GameState::CustomBattle,
            vec![
                GameState::Loaded,
                GameState::CustomBattle,
                GameState::Battle,
            ],
        );
        m.insert(GameState::Shop, vec![GameState::Loaded, GameState::Shop]);
        m.insert(
            GameState::TestScenariosRun,
            vec![GameState::Loaded, GameState::TestScenariosLoad],
        );
        m.insert(
            GameState::ServerSync,
            vec![GameState::Loaded, GameState::Connect, GameState::ServerSync],
        );
        m.insert(
            GameState::Profile,
            vec![
                GameState::Loaded,
                GameState::Connect,
                GameState::Login,
                GameState::Profile,
            ],
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
    pub fn run_to_target(self, world: &mut World) {
        self.set_target();
        GameState::Loaded.change(world);
    }
    pub fn change(self, world: &mut World) {
        info!("State change to {self}");
        Self::set_ready(true);
        world.resource_mut::<NextState<GameState>>().set(self);
    }
    pub fn set_ready(value: bool) {
        *STATE_READY.lock().unwrap() = value;
    }
    pub fn is_ready() -> bool {
        *STATE_READY.lock().unwrap()
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
    fn on_changed(state: Res<State<GameState>>, mut next_state: ResMut<NextState<GameState>>) {
        let state = state.get();
        info!("Enter state: {state}");
        Self::proceed(*state, next_state.as_mut());
    }
    fn proceed(state: GameState, next_state: &mut NextState<GameState>) {
        if !GameState::is_ready() {
            info!("Game state not ready");
            return;
        }
        if let Some(path) = STATE_PATHS.get(&GameState::get_target()) {
            if state.eq(path.last().unwrap()) {
                return;
            }
            for i in 0..path.len() {
                if state.eq(&path[i]) {
                    info!("Path proceed from {state} to {}", path[i + 1]);
                    next_state.set(path[i + 1]);
                }
            }
        }
    }
    pub fn path_proceed(world: &mut World) {
        GameState::set_ready(true);
        Self::proceed(
            cur_state(world),
            world.resource_mut::<NextState<GameState>>().as_mut(),
        );
    }
    fn update(time: Res<Time>) {
        let mut timer = GameTimer::get();
        timer.update(time.delta_seconds());
    }
}
