use super::*;

#[derive(Clone, Copy, Eq, PartialEq, Debug, Hash, Default, States, Display)]
pub enum GameState {
    #[default]
    Loading,
    Loaded,
    Title,
    MetaShop,
    MetaHeroes,
    MetaHeroShards,
    MetaLootboxes,
    Inbox,
    Connect,
    Login,
    CustomBattle,
    ShopBattle,
    Battle,
    Shop,
    GameStart,
    GameOver,
    TestScenariosLoad,
    TestScenariosRun,
    ServerSync,
    GameArchiveDownload,
    GameArchiveUpload,
    Profile,
    TableView(StdbQuery),
    Error,
    Teams,
    TeamEditor,
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
        m.insert(
            GameState::TableView(StdbQuery::BaseUnits),
            [GameOption::Connect, GameOption::Table(StdbQuery::BaseUnits)].into(),
        );
        m.insert(
            GameState::TableView(StdbQuery::BattleHistory),
            [
                GameOption::Connect,
                GameOption::Table(StdbQuery::BattleHistory),
            ]
            .into(),
        );
        m.insert(GameState::GameArchiveUpload, [GameOption::Connect].into());
        m.insert(
            GameState::GameArchiveDownload,
            [GameOption::Connect, GameOption::Table(StdbQuery::GameFull)].into(),
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
    pub fn get_name(self) -> &'static str {
        match self {
            GameState::MetaShop => "Shop",
            GameState::MetaHeroes => "Heroes",
            GameState::MetaHeroShards => "Hero Shards",
            GameState::MetaLootboxes => "Lootboxes",
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
    let to = cur_state(world);
    TeamSyncPlugin::unsubscribe_all(world);
    TilePlugin::change_state(to, world);
}

lazy_static! {
    static ref ENTITY_NAMES: Mutex<HashMap<Entity, Cstr>> = Mutex::new(HashMap::new());
}

pub fn save_entity_name(entity: Entity, name: Cstr) {
    ENTITY_NAMES.lock().unwrap().insert(entity, name);
}
pub fn entity_name(entity: Entity) -> Cstr {
    ENTITY_NAMES
        .lock()
        .unwrap()
        .get(&entity)
        .cloned()
        .unwrap_or_default()
}
pub fn entity_name_with_id(entity: Entity) -> Cstr {
    entity_name(entity).push(format!("#{entity}").cstr()).take()
}
pub fn clear_entity_names() {
    ENTITY_NAMES.lock().unwrap().clear();
}
