use egui_tiles::Container;

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
    Shop,
    Battle,
    MatchOver,
    FusionEditor,
    TestScenariosLoad,
    TestScenariosRun,
    ServerSync,
    MigrationDownload,
    MigrationUpload,
    WorldDownload,
    WorldUpload,
    Error,
    Query,
    Editor,
    Explorer,
}

const TREE_ID: &str = "tree";
impl GameState {
    pub fn load_tree(self, tile_tree: &mut TileTree) {
        match self {
            GameState::Connect => {
                tile_tree.tree = Tree::new_tabs(TREE_ID, Pane::Connect.into());
            }
            GameState::Login => {
                tile_tree.tree = Tree::new_tabs(TREE_ID, Pane::Login.into());
            }
            GameState::Register => {
                tile_tree.tree = Tree::new_tabs(TREE_ID, Pane::Register.into());
            }
            GameState::Title => {
                let mut tiles = Tiles::default();
                let leaderboard = tiles.insert_pane(Pane::Leaderboard);
                let history = tiles.insert_pane(Pane::BattleHistory);
                let main_menu = tiles.insert_pane(Pane::MainMenu);
                let tabs = tiles.insert_tab_tile([leaderboard, history].into());
                let mut horizontal = [main_menu, tabs].to_vec();
                if pd().client_settings.dev_mode {
                    horizontal.insert(0, tiles.insert_pane(Pane::Admin));
                }
                let root = tiles.insert_horizontal_tile(horizontal);
                tile_tree.tree = Tree::new(TREE_ID, root, tiles);
            }
            GameState::MatchOver => {
                tile_tree.tree = Tree::new_tabs(TREE_ID, Pane::MatchOver.into());
            }
            GameState::Editor => {
                let mut tiles = Tiles::default();
                let view = tiles.insert_pane(Pane::Battle(BattlePane::View));

                // Left team graph
                let left_team_graph = tiles.insert_pane(Pane::Battle(BattlePane::EditLeftGraph));

                // Right team graph
                let right_team_graph = tiles.insert_pane(Pane::Battle(BattlePane::EditRightGraph));

                let team_tabs = tiles.insert_tab_tile([left_team_graph, right_team_graph].into());
                let root = tiles.insert_vertical_tile([view, team_tabs].into());
                if let Tile::Container(h) = tiles.get_mut(root).unwrap() {
                    if let Container::Linear(h) = h {
                        h.shares.set_share(view, 1.0);
                        h.shares.set_share(team_tabs, 2.5);
                    }
                }
                tile_tree.tree = Tree::new(TREE_ID, root, tiles);
            }
            GameState::Explorer => {
                let mut tiles = Tiles::default();

                // Create Units tab layout
                let units_list = tiles.insert_pane(Pane::Explorer(ExplorerPane::UnitsList));
                let unit_card = tiles.insert_pane(Pane::Explorer(ExplorerPane::UnitCard));
                let left_column = tiles.insert_vertical_tile([units_list, unit_card].into());

                // Create 2x2 component panes for units
                let unit_description =
                    tiles.insert_pane(Pane::Explorer(ExplorerPane::UnitDescription));
                let unit_behavior = tiles.insert_pane(Pane::Explorer(ExplorerPane::UnitBehavior));
                let unit_representation =
                    tiles.insert_pane(Pane::Explorer(ExplorerPane::UnitRepresentation));
                let unit_stats = tiles.insert_pane(Pane::Explorer(ExplorerPane::UnitStats));

                let middle_column =
                    tiles.insert_vertical_tile([unit_description, unit_behavior].into());
                let right_column =
                    tiles.insert_vertical_tile([unit_representation, unit_stats].into());

                let units_content =
                    tiles.insert_horizontal_tile([left_column, middle_column, right_column].into());
                if let Tile::Container(h) = tiles.get_mut(units_content).unwrap() {
                    if let Container::Linear(h) = h {
                        h.shares.set_share(middle_column, 2.0);
                        h.shares.set_share(right_column, 2.0);
                    }
                }

                let units_tab = units_content.with_name(tile_tree, "Units");

                // Create Houses tab layout
                let houses_list = tiles.insert_pane(Pane::Explorer(ExplorerPane::HousesList));
                let house_card = tiles.insert_pane(Pane::Explorer(ExplorerPane::HouseCard));
                let houses_left_column =
                    tiles.insert_vertical_tile([houses_list, house_card].into());

                // Create 2 rows for abilities and statuses
                let ability_magic = tiles.insert_pane(Pane::Explorer(ExplorerPane::AbilityMagic));
                let ability_description =
                    tiles.insert_pane(Pane::Explorer(ExplorerPane::AbilityDescription));
                let ability_effect = tiles.insert_pane(Pane::Explorer(ExplorerPane::AbilityEffect));
                let abilities_row = tiles.insert_horizontal_tile(
                    [ability_magic, ability_description, ability_effect].into(),
                );

                let status_magic = tiles.insert_pane(Pane::Explorer(ExplorerPane::StatusMagic));
                let status_description =
                    tiles.insert_pane(Pane::Explorer(ExplorerPane::StatusDescription));
                let status_behavior =
                    tiles.insert_pane(Pane::Explorer(ExplorerPane::StatusBehavior));
                let statuses_row = tiles.insert_horizontal_tile(
                    [status_magic, status_description, status_behavior].into(),
                );

                let right_content =
                    tiles.insert_vertical_tile([abilities_row, statuses_row].into());
                let house_color = tiles.insert_pane(Pane::Explorer(ExplorerPane::HouseColor));

                let houses_content = tiles.insert_horizontal_tile(
                    [houses_left_column, right_content, house_color].into(),
                );
                if let Tile::Container(h) = tiles.get_mut(houses_content).unwrap() {
                    if let Container::Linear(h) = h {
                        h.shares.set_share(right_content, 3.0);
                    }
                }
                if let Tile::Container(h) = tiles.get_mut(houses_content).unwrap() {
                    if let Container::Linear(h) = h {
                        h.shares.set_share(house_color, 0.6);
                    }
                }

                let houses_tab = houses_content.with_name(tile_tree, "Houses");

                // Main tabs - only Units and Houses
                let root = tiles.insert_tab_tile([units_tab, houses_tab].into());
                tile_tree.tree = Tree::new(TREE_ID, root, tiles);
            }
            GameState::Shop => {
                let mut tiles = Tiles::default();
                let shop = tiles.insert_pane(Pane::Shop(ShopPane::Shop));
                let info = tiles.insert_pane(Pane::Shop(ShopPane::Info));
                let team = tiles.insert_pane(Pane::Shop(ShopPane::Team));
                let right = tiles.insert_vertical_tile([info, shop].into());
                if let Tile::Container(h) = tiles.get_mut(right).unwrap() {
                    if let Container::Linear(h) = h {
                        h.shares.set_share(shop, 4.5);
                    }
                }
                let root = tiles.insert_horizontal_tile([team, right].into());
                if let Tile::Container(h) = tiles.get_mut(root).unwrap() {
                    if let Container::Linear(h) = h {
                        h.shares.set_share(team, 2.0);
                    }
                }
                tile_tree.tree = Tree::new(TREE_ID, root, tiles);
            }
            GameState::Battle => {
                tile_tree.tree = Tree::new_tabs(TREE_ID, Pane::Battle(BattlePane::View).into());
            }
            GameState::WorldDownload => {
                tile_tree.tree = Tree::new_tabs(TREE_ID, Pane::WorldDownload.into());
            }
            _ => {
                tile_tree.tree = Tree::empty(TREE_ID);
            }
        };
    }
}

#[derive(PartialEq, Eq, Clone, Hash, AsRefStr, Serialize, Deserialize, Debug, Display, Copy)]
pub enum Pane {
    Connect,
    Login,
    Register,
    MainMenu,
    Battle(BattlePane),
    Shop(ShopPane),
    Explorer(ExplorerPane),
    MatchOver,
    Leaderboard,
    BattleHistory,
    Admin,
    WorldDownload,
}

#[derive(PartialEq, Eq, Clone, Copy, Hash, AsRefStr, Serialize, Deserialize, Debug, Display)]
pub enum BattlePane {
    View,
    EditLeftGraph,
    EditRightGraph,
}
#[derive(PartialEq, Eq, Clone, Copy, Hash, AsRefStr, Serialize, Deserialize, Debug, Display)]
pub enum ShopPane {
    Shop,
    Roster,
    Team,
    Info,
}

impl Into<Vec<Pane>> for Pane {
    fn into(self) -> Vec<Pane> {
        [self].into()
    }
}

impl Pane {
    pub fn ui(self, ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        match self {
            Pane::MainMenu => {
                ui.vertical_centered_justified(|ui| {
                    ui.add_space(ui.available_height() * 0.3);
                    ui.set_width(350.0.at_most(ui.available_width()));
                    with_solid_source(|ctx| {
                        match player(ctx)?.active_match_ref(ctx) {
                            Ok(_) => {
                                if "Abandon Game"
                                    .cstr_cs(RED, CstrStyle::Bold)
                                    .button(ui)
                                    .clicked()
                                {
                                    Confirmation::new("Abandon current run?")
                                        .accept_name("[red Abandon]")
                                        .content(|_ui, world, button_pressed| {
                                            if let Some(true) = button_pressed {
                                                cn().reducers.match_abandon().notify_error(world);
                                            }
                                        })
                                        .push(world);
                                }
                                ui.add_space(20.0);
                                ui.separator();
                                ui.add_space(20.0);
                                if "Continue"
                                    .cstr_cs(high_contrast_text(), CstrStyle::Heading2)
                                    .button(ui)
                                    .clicked()
                                {
                                    GameState::Shop.set_next(world);
                                }
                            }
                            Err(_) => {
                                if "New Game"
                                    .cstr_cs(high_contrast_text(), CstrStyle::Heading2)
                                    .button(ui)
                                    .clicked()
                                {
                                    cn().reducers.match_insert().notify_error_op();
                                    static ON_MATCH_CALLBACK: Mutex<Option<MatchInsertCallbackId>> =
                                        Mutex::new(None);
                                    let callback_id = cn().reducers.on_match_insert(|e| {
                                        if !e.check_identity() {
                                            return;
                                        }
                                        if matches!(
                                            e.event.status,
                                            spacetimedb_sdk::Status::Committed
                                        ) {
                                            if let Some(callback_id) =
                                                ON_MATCH_CALLBACK.lock().take()
                                            {
                                                debug!("removed callback");
                                                cn().reducers.remove_on_match_insert(callback_id);
                                            }
                                            GameState::Shop.set_next_op();
                                        }
                                    });
                                    *ON_MATCH_CALLBACK.lock() = Some(callback_id);
                                }
                            }
                        }
                        Ok(())
                    })
                    .ui(ui);
                });
            }
            Pane::Login => LoginPlugin::pane_login(ui, world),
            Pane::Register => LoginPlugin::pane_register(ui, world),
            Pane::Connect => ConnectPlugin::pane(ui),
            Pane::Admin => AdminPlugin::pane(ui, world),
            Pane::Leaderboard => MatchPlugin::pane_leaderboard(ui, world)?,
            Pane::BattleHistory => MatchPlugin::pane_battle_history(ui, world)?,
            Pane::MatchOver => MatchPlugin::pane_match_over(ui, world)?,
            Pane::Shop(pane) => match pane {
                ShopPane::Shop => MatchPlugin::pane_shop(ui, world)?,
                ShopPane::Info => MatchPlugin::pane_info(ui, world)?,
                ShopPane::Roster => MatchPlugin::pane_roster(ui, world)?,
                ShopPane::Team => MatchPlugin::pane_team(ui, world)?,
            },
            Pane::Battle(pane) => match pane {
                BattlePane::View => BattleEditorPlugin::pane_view(ui, world),
                BattlePane::EditLeftGraph => BattleEditorPlugin::pane_edit_graph(true, ui, world),
                BattlePane::EditRightGraph => BattleEditorPlugin::pane_edit_graph(false, ui, world),
            },
            Pane::Explorer(pane) => ExplorerPlugin::pane(pane, ui, world)?,
            Pane::WorldDownload => world_download_ui_system(ui, world),
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
    m.insert(GameState::MigrationUpload, [GameOption::Connect].into());
    m.insert(GameState::MigrationDownload, [GameOption::Connect].into());
    m.insert(
        GameState::WorldUpload,
        [GameOption::Connect, GameOption::ForceTablesSubscribe].into(),
    );
    m.insert(
        GameState::WorldDownload,
        [GameOption::Connect, GameOption::ForceTablesSubscribe].into(),
    );
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
        op(move |world| self.set_next(world));
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
        op(Self::proceed);
    }
    pub fn proceed_to_target(self, world: &mut World) {
        self.set_target();
        Self::proceed(world);
    }
    pub fn proceed_to_target_op(self) {
        op(move |world| self.proceed_to_target(world))
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
