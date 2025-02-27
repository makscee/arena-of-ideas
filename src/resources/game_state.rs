use super::*;

#[derive(
    Clone, Copy, Eq, PartialEq, Debug, Hash, Default, States, Display, Deserialize, Serialize,
)]
pub enum GameState {
    #[default]
    Loading,
    Loaded,
    Title,
    Connect,
    Login,
    Register,
    Match,
    FusionEditor,
    TestScenariosLoad,
    TestScenariosRun,
    ServerSync,
    MigrationDownload,
    MigrationUpload,
    Error,
    Query,
}

impl GameState {
    pub fn load_state(self) -> DockState<Tab> {
        match self {
            GameState::Connect => DockState::new(Tab::Login.into()),
            GameState::Login => DockState::new(Tab::Login.into()),
            GameState::Register => DockState::new(Tab::Register.into()),
            GameState::Title => DockState::new([Tab::MainMenu, Tab::Admin].into()),
            GameState::Match => {
                let mut ds = DockState::new([Tab::Shop].into());
                ds.main_surface_mut()
                    .split_below(0.into(), 0.5, Tab::Team.into());
                ds.main_surface_mut()
                    .split_left(0.into(), 0.15, Tab::Roster.into());
                ds
            }
            GameState::FusionEditor => {
                let mut ds = DockState::new([Tab::Roster].into());
                ds.main_surface_mut()
                    .split_right(0.into(), 0.5, Tab::Triggers.into());
                ds.main_surface_mut()
                    .split_right(0.into(), 0.5, Tab::Actions.into());
                ds.main_surface_mut()
                    .split_below(0.into(), 0.5, Tab::FusionResult.into());
                ds
            }
            _ => DockState::new(default()),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Hash, AsRefStr, Serialize, Deserialize, Debug)]
pub enum Tab {
    Connect,
    Login,
    Register,
    MainMenu,
    Shop,
    Team,
    Roster,
    Triggers,
    Actions,
    FusionResult,

    Admin,
}

impl Tab {
    pub fn ui(&self, ui: &mut Ui, world: &mut World) {
        match self {
            Tab::MainMenu => {
                ui.vertical_centered_justified(|ui| {
                    ui.add_space(ui.available_height() * 0.3);
                    ui.set_width(350.0.at_most(ui.available_width()));
                    if "Open Match"
                        .cstr_cs(VISIBLE_LIGHT, CstrStyle::Bold)
                        .button(ui)
                        .clicked()
                    {
                        GameState::Match.set_next(world);
                    }
                });
            }
            Tab::Login => LoginPlugin::tab_login(ui, world),
            Tab::Register => LoginPlugin::tab_register(ui, world),
            Tab::Connect => ConnectPlugin::tab(ui),
            Tab::Admin => AdminPlugin::tab(ui, world),
            Tab::Shop => MatchPlugin::shop_tab(ui, world),
            Tab::Roster => match cur_state(world) {
                GameState::Match => MatchPlugin::roster_tab(ui, world),
                GameState::FusionEditor => FusionEditorPlugin::roster_tab(ui, world),
                _ => unreachable!(),
            },
            Tab::Triggers => FusionEditorPlugin::triggers_tab(ui, world),
            Tab::Actions => FusionEditorPlugin::actions_tab(ui, world),
            Tab::FusionResult => FusionEditorPlugin::fusion_result_tab(ui, world),
            Tab::Team => MatchPlugin::team_tab(ui, world),
        }
    }
}

impl ToCstr for GameState {
    fn cstr(&self) -> Cstr {
        self.to_string().cstr_cs(GREEN, CstrStyle::Bold)
    }
}

const STATE_OPTIONS: LazyCell<HashMap<GameState, Vec<GameOption>>> = LazyCell::new(|| {
    let mut m = HashMap::new();
    m.insert(
        GameState::Title,
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
    m
});

static TARGET_STATE: Mutex<GameState> = Mutex::new(GameState::Loaded);

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
            self.cstr().to_colored()
        );
        world.resource_mut::<NextState<GameState>>().set(self);
    }
    pub fn set_next_op(self) {
        OperationsPlugin::add(move |world| self.set_next(world));
    }
    pub fn proceed(world: &mut World) {
        let to = *TARGET_STATE.lock();
        info!("Proceed to {to}");
        if let Some(options) = STATE_OPTIONS.get(&to) {
            for option in options {
                if !option.is_fulfilled(world) {
                    option.fulfill(world);
                    return;
                }
            }
        }
        to.set_next(world);
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
        self.to_string().to_case(Case::Title).leak()
    }
}

pub struct GameStatePlugin;

impl Plugin for GameStatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Loaded), GameState::proceed)
            .add_systems(Update, on_change.run_if(state_changed::<GameState>));
    }
}

static PREVIOUS_STATE: Mutex<GameState> = Mutex::new(GameState::Loading);
fn on_change(world: &mut World) {
    if let Some(ctx) = egui_context(world) {
        ctx.data_mut(|w| w.clear());
    }
    let from = *PREVIOUS_STATE.lock();
    let to = cur_state(world);
    *PREVIOUS_STATE.lock() = to;
    info!(
        "State change {} -> {}",
        from.cstr().to_colored(),
        to.cstr().to_colored()
    );
    DockPlugin::load_state_tree(from, to, world);

    // TilePlugin::change_state(to, world);
    // CameraPlugin::respawn_camera(world);
}
