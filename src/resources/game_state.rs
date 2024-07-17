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
    ShopBattle,
    Battle,
    Shop,
    GameOver,
    TestScenariosLoad,
    TestScenariosRun,
    ServerSync,
    Profile,
    TableView(&'static str),
}

static TARGET_STATE: Mutex<GameState> = Mutex::new(GameState::Loaded);

lazy_static! {
    static ref STATE_OPTIONS: HashMap<GameState, Vec<GameOption>> = {
        let mut m = HashMap::new();
        m.insert(
            GameState::Title,
            [GameOption::Connect, GameOption::Login].into(),
        );
        m.insert(
            GameState::Profile,
            [GameOption::Connect, GameOption::Login].into(),
        );
        m.insert(
            GameState::Shop,
            [GameOption::Connect, GameOption::ForceLogin].into(),
        );
        m.insert(GameState::ServerSync, [GameOption::Connect].into());
        m.insert(
            GameState::TestScenariosRun,
            [GameOption::TestScenariosLoad].into(),
        );
        m.insert(
            GameState::TableView(QUERY_LEADERBOARD),
            [GameOption::Connect, GameOption::Table(QUERY_LEADERBOARD)].into(),
        );
        m.insert(
            GameState::TableView(QUERY_BASE_UNITS),
            [GameOption::Connect, GameOption::Table(QUERY_BASE_UNITS)].into(),
        );
        m.insert(
            GameState::TableView(QUERY_BATTLE_HISTORY),
            [
                GameOption::Connect,
                GameOption::ForceLogin,
                GameOption::Table(QUERY_BATTLE_HISTORY),
            ]
            .into(),
        );
        m
    };
}

impl GameState {
    pub fn get_target() -> Self {
        *TARGET_STATE.lock().unwrap()
    }
    pub fn set_target(self) {
        *TARGET_STATE.lock().unwrap() = self;
    }
    pub fn set_next(self, world: &mut World) {
        info!(
            "{} {}",
            "Set next state:".dimmed(),
            self.to_string().bold().green()
        );
        world.resource_mut::<NextState<GameState>>().set(self);
    }
    pub fn set_next_op(self) {
        OperationsPlugin::add(move |world| self.set_next(world));
    }
    pub fn proceed(world: &mut World) {
        let target = *TARGET_STATE.lock().unwrap();
        if let Some(options) = STATE_OPTIONS.get(&target) {
            for option in options {
                if !option.is_fulfilled(world) {
                    option.fulfill(world);
                    return;
                }
            }
        }
        target.set_next(world);
    }
    pub fn proceed_op() {
        OperationsPlugin::add(Self::proceed);
    }
    pub fn proceed_to_target(self, world: &mut World) {
        self.set_target();
        Self::proceed(world);
    }
    pub fn proceed_to_target_op(self) {
        OperationsPlugin::add(move |world| self.proceed_to_target(world))
    }
}

pub struct GameStatePlugin;

impl Plugin for GameStatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Loaded), GameState::proceed);
    }
}
