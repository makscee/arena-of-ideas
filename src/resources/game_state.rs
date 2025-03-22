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
    Incubator,
    TestScenariosLoad,
    TestScenariosRun,
    ServerSync,
    MigrationDownload,
    MigrationUpload,
    Error,
    Query,
}

const TREE_ID: &str = "tree";
impl GameState {
    pub fn load_tree(self) -> Tree<Pane> {
        match self {
            GameState::Connect => Tree::new_tabs(TREE_ID, Pane::Connect.into()),
            GameState::Login => Tree::new_tabs(TREE_ID, Pane::Login.into()),
            GameState::Register => Tree::new_tabs(TREE_ID, Pane::Register.into()),
            GameState::Title => Tree::new_horizontal(TREE_ID, [Pane::Admin, Pane::MainMenu].into()),
            GameState::Incubator => {
                let mut tiles = Tiles::default();
                let left = [
                    tiles.insert_pane(Pane::IncubatorInspect),
                    tiles.insert_pane(Pane::IncubatorNewNode),
                ]
                .into();
                let left = tiles.insert_tab_tile(left);
                let right = tiles.insert_pane(Pane::IncubatorNodes);
                let root = tiles.insert_horizontal_tile([left, right].into());
                Tree::new(TREE_ID, root, tiles)
            }
            _ => Tree::empty(TREE_ID),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Hash, AsRefStr, Serialize, Deserialize, Debug, Display)]
pub enum Pane {
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
    BattleEditor,

    IncubatorNodes,
    IncubatorNewNode,
    IncubatorInspect,

    BattleView,
    BattleControls,

    EditorTeam,

    Admin,
    WorldInspector,
    NodeGraph,
}

impl Into<Vec<Pane>> for Pane {
    fn into(self) -> Vec<Pane> {
        [self].into()
    }
}

impl Pane {
    pub fn ui(self, ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        match self {
            Pane::MainMenu => {
                ui.vertical_centered_justified(|ui| {
                    ui.add_space(ui.available_height() * 0.3);
                    ui.set_width(350.0.at_most(ui.available_width()));
                    if "Open Match"
                        .cstr_cs(tokens_global().high_contrast_text(), CstrStyle::Bold)
                        .button(ui)
                        .clicked()
                    {
                        GameState::Match.set_next(world);
                    }
                });
            }
            Pane::Login => LoginPlugin::pane_login(ui, world),
            Pane::Register => LoginPlugin::pane_register(ui, world),
            Pane::Connect => ConnectPlugin::pane(ui),
            Pane::Admin => AdminPlugin::pane(ui, world),
            Pane::Shop => MatchPlugin::pane_shop(ui, world)?,
            Pane::Roster => match cur_state(world) {
                GameState::Match => MatchPlugin::pane_roster(ui, world)?,
                GameState::FusionEditor => FusionEditorPlugin::pane_roster(ui, world)?,
                _ => unreachable!(),
            },
            Pane::Team => MatchPlugin::pane_team(ui, world)?,
            Pane::Triggers => FusionEditorPlugin::pane_triggers(ui, world),
            Pane::Actions => FusionEditorPlugin::pane_actions(ui, world),
            Pane::FusionResult => FusionEditorPlugin::pane_fusion_result(ui, world)?,
            Pane::BattleEditor => BattleEditorPlugin::pane(ui, world)?,

            Pane::IncubatorNewNode => IncubatorPlugin::pane_new_node(ui, world)?,
            Pane::IncubatorInspect => IncubatorPlugin::pane_inspect(ui, world)?,
            Pane::IncubatorNodes => IncubatorPlugin::pane_nodes(ui, world)?,

            Pane::BattleView => BattlePlugin::pane_view(ui, world)?,
            Pane::BattleControls => BattlePlugin::pane_controls(ui, world)?,

            Pane::EditorTeam => Team::show_editor(self, ui, world),

            Pane::WorldInspector => bevy_inspector_egui::bevy_inspector::ui_for_world(world, ui),
            Pane::NodeGraph => NodeGraphPlugin::pane_ui(ui, world),
        };
        Ok(())
    }
}

impl ToCstr for GameState {
    fn cstr(&self) -> Cstr {
        self.to_string().cstr_cs(GREEN, CstrStyle::Bold)
    }
}

const STATE_OPTIONS: LazyCell<HashMap<GameState, Vec<GameOption>>> = LazyCell::new(|| {
    let mut m = HashMap::new();
    m.insert(GameState::Login, [GameOption::Connect].into());
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
    TilePlugin::load_state_tree(to, world);
}
