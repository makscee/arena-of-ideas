use super::*;

#[derive(
    Clone, Copy, Eq, PartialEq, Debug, Hash, Default, States, Display, Deserialize, Serialize,
)]
pub enum GameState {
    #[default]
    Loading,
    Loaded,
    Title,
    Meta,
    Inbox,
    Connect,
    Login,
    CustomBattle,
    Battle,
    Match,
    GameStart,
    GameOver,
    TestScenariosLoad,
    TestScenariosRun,
    ServerSync,
    MigrationDownload,
    MigrationUpload,
    Profile,
    Error,
    Editor,
    Quests,
    Stats,
    Incubator,
    Players,
    Query,
    Admin,
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
            GameState::Match,
            [
                GameOption::Connect,
                GameOption::ForceLogin,
                GameOption::ActiveRun,
            ]
            .into(),
        );
        m.insert(GameState::ServerSync, [GameOption::Connect].into());
        m.insert(
            GameState::TestScenariosRun,
            [GameOption::TestScenariosLoad].into(),
        );
        m.insert(GameState::MigrationUpload, [GameOption::Connect].into());
        m.insert(GameState::MigrationDownload, [GameOption::Connect].into());
        m.insert(
            GameState::Admin,
            [GameOption::Connect, GameOption::Login].into(),
        );
        m.insert(
            GameState::Incubator,
            [GameOption::Connect, GameOption::ForceLogin].into(),
        );
        m
    };
}

impl GameState {
    pub fn get_target() -> Self {
        *TARGET_STATE.lock()
    }
    pub fn set_target(self) {
        *TARGET_STATE.lock() = self;
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
        let target = *TARGET_STATE.lock();
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
    pub fn get_name(self) -> &'static str {
        match self {
            _ => self.to_string().to_case(Case::Title).leak(),
        }
    }
}

pub struct GameStatePlugin;

impl Plugin for GameStatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Loaded), GameState::proceed)
            .add_systems(Update, on_change.run_if(state_changed::<GameState>));
    }
}

fn on_change(world: &mut World) {
    if let Some(ctx) = egui_context(world) {
        ctx.data_mut(|w| w.clear());
    }
    // let to = cur_state(world);
    // TilePlugin::change_state(to, world);
    // CameraPlugin::respawn_camera(world);
}

lazy_static! {
    static ref ENTITY_NAMES: Mutex<HashMap<Entity, Cstr>> = Mutex::new(HashMap::new());
}

pub fn save_entity_name(entity: Entity, name: Cstr) {
    ENTITY_NAMES.lock().insert(entity, name);
}
pub fn entity_name(entity: Entity) -> String {
    ENTITY_NAMES
        .lock()
        .get(&entity)
        .cloned()
        .unwrap_or(entity.to_string().cstr())
}
pub fn entity_name_with_id(entity: Entity) -> Cstr {
    entity_name(entity) + &format!("#{entity}")
}
pub fn clear_entity_names() {
    ENTITY_NAMES.lock().clear();
}
