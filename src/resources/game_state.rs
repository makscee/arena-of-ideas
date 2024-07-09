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
        info!("Set next state: {self}");
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
