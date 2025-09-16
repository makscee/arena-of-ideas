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
                tile_tree.tree = Tree::new_horizontal(
                    TREE_ID,
                    [Pane::Admin, Pane::MainMenu, Pane::Leaderboard].into(),
                );
            }
            GameState::MatchOver => {
                tile_tree.tree = Tree::new_tabs(TREE_ID, Pane::MatchOver.into());
            }
            GameState::Editor => {
                let mut tiles = Tiles::default();
                let view = tiles.insert_pane(Pane::Battle(BattlePane::View));
                let edit_left_graph = tiles.insert_pane(Pane::Battle(BattlePane::EditLeftGraph));
                let edit_right_graph = tiles.insert_pane(Pane::Battle(BattlePane::EditRightGraph));
                let battle_editor = tiles.insert_pane(Pane::Battle(BattlePane::BattleEditor));
                let edit_left = tiles.insert_vertical_tile([edit_left_graph].into());
                let edit_right = tiles.insert_vertical_tile([edit_right_graph].into());
                let edit = tiles.insert_tab_tile([edit_left, edit_right, battle_editor].into());
                let root = tiles.insert_vertical_tile([view, edit].into());
                tile_tree.tree = Tree::new(TREE_ID, root, tiles);
            }
            GameState::Explorer => {
                let tiles = &mut Tiles::default();

                // Units tab
                let units_list = tiles.insert_pane(Pane::Explorer(ExplorerPane::Units));
                let units_houses_list =
                    tiles.insert_pane(Pane::Explorer(ExplorerPane::UnitsHouses));
                let units_left_column =
                    tiles.insert_vertical_tile([units_list, units_houses_list].into());

                let representations_pane =
                    tiles.insert_pane(Pane::Explorer(ExplorerPane::Representations));
                let descriptions_pane =
                    tiles.insert_pane(Pane::Explorer(ExplorerPane::Descriptions));
                let behaviors_pane = tiles.insert_pane(Pane::Explorer(ExplorerPane::Behaviors));
                let stats_pane = tiles.insert_pane(Pane::Explorer(ExplorerPane::Stats));

                let units_top_row =
                    tiles.insert_horizontal_tile([representations_pane, descriptions_pane].into());
                let units_bottom_row =
                    tiles.insert_horizontal_tile([behaviors_pane, stats_pane].into());
                let units_right_grid =
                    tiles.insert_vertical_tile([units_top_row, units_bottom_row].into());

                let units_tab =
                    tiles.insert_horizontal_tile([units_left_column, units_right_grid].into());
                if let Tile::Container(container) = tiles.get_mut(units_tab).unwrap() {
                    if let Container::Linear(linear) = container {
                        linear.shares.set_share(units_left_column, 1.0);
                        linear.shares.set_share(units_right_grid, 2.0);
                    }
                }

                // Houses tab
                let houses_list = tiles.insert_pane(Pane::Explorer(ExplorerPane::Houses));
                let houses_units_list =
                    tiles.insert_pane(Pane::Explorer(ExplorerPane::HousesUnits));
                let houses_left_column =
                    tiles.insert_vertical_tile([houses_list, houses_units_list].into());

                let house_colors_pane =
                    tiles.insert_pane(Pane::Explorer(ExplorerPane::HouseColors));
                let ability_magic_pane =
                    tiles.insert_pane(Pane::Explorer(ExplorerPane::AbilityMagic));
                let status_magic_pane =
                    tiles.insert_pane(Pane::Explorer(ExplorerPane::StatusMagic));

                let houses_top_row =
                    tiles.insert_horizontal_tile([house_colors_pane, ability_magic_pane].into());
                let houses_bottom_row = tiles.insert_horizontal_tile([status_magic_pane].into());
                let houses_right_grid =
                    tiles.insert_vertical_tile([houses_top_row, houses_bottom_row].into());

                let houses_tab =
                    tiles.insert_horizontal_tile([houses_left_column, houses_right_grid].into());
                if let Tile::Container(container) = tiles.get_mut(houses_tab).unwrap() {
                    if let Container::Linear(linear) = container {
                        linear.shares.set_share(houses_left_column, 1.0);
                        linear.shares.set_share(houses_right_grid, 2.0);
                    }
                }

                // Create tab container
                let root = tiles.insert_tab_tile([units_tab, houses_tab].into());
                tile_tree
                    .behavior
                    .tile_names
                    .insert(units_tab, "Units".to_string());
                tile_tree
                    .behavior
                    .tile_names
                    .insert(houses_tab, "Houses".to_string());

                tile_tree.tree = Tree::new(TREE_ID, root, mem::take(tiles));
            }
            GameState::Shop => {
                let mut tiles = Tiles::default();
                let shop = tiles.insert_pane(Pane::Shop(ShopPane::Shop));
                let info = tiles.insert_pane(Pane::Shop(ShopPane::Info));
                let team = tiles.insert_pane(Pane::Shop(ShopPane::Team));
                let top = tiles.insert_horizontal_tile([shop, info].into());
                if let Tile::Container(h) = tiles.get_mut(top).unwrap() {
                    if let Container::Linear(h) = h {
                        h.shares.set_share(shop, 4.0);
                    }
                }
                let root = tiles.insert_vertical_tile([top, team].into());
                tile_tree.tree = Tree::new(TREE_ID, root, tiles);
            }
            GameState::Battle => {
                tile_tree.tree = Tree::new_tabs(TREE_ID, Pane::Battle(BattlePane::View).into());
            }
            _ => {
                tile_tree.tree = Tree::empty(TREE_ID);
            }
        };
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Hash, AsRefStr, Serialize, Deserialize, Debug, Display)]
pub enum Pane {
    Connect,
    Login,
    Register,
    MainMenu,

    Battle(BattlePane),
    Shop(ShopPane),
    MatchOver,
    Leaderboard,

    Admin,
    Explorer(ExplorerPane),
}

#[derive(PartialEq, Eq, Clone, Copy, Hash, AsRefStr, Serialize, Deserialize, Debug, Display)]
pub enum BattlePane {
    View,
    EditLeftGraph,
    EditRightGraph,
    BattleEditor,
}
#[derive(PartialEq, Eq, Clone, Copy, Hash, AsRefStr, Serialize, Deserialize, Debug, Display)]
pub enum ShopPane {
    Shop,
    Roster,
    Team,
    Info,
    Fusion,
}

#[derive(PartialEq, Eq, Clone, Copy, Hash, AsRefStr, Serialize, Deserialize, Debug, Display)]
pub enum ExplorerPane {
    Units,
    UnitsHouses,
    Representations,
    Descriptions,
    Behaviors,
    Stats,
    Houses,
    HousesUnits,
    HouseColors,
    AbilityMagic,
    StatusMagic,
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
                        .cstr_cs(high_contrast_text(), CstrStyle::Bold)
                        .button(ui)
                        .clicked()
                    {
                        GameState::Shop.set_next(world);
                    }
                });
            }
            Pane::Login => LoginPlugin::pane_login(ui, world),
            Pane::Register => LoginPlugin::pane_register(ui, world),
            Pane::Connect => ConnectPlugin::pane(ui),
            Pane::Admin => AdminPlugin::pane(ui, world),
            Pane::Leaderboard => MatchPlugin::pane_leaderboard(ui, world)?,
            Pane::MatchOver => MatchPlugin::pane_match_over(ui, world)?,
            Pane::Shop(pane) => match pane {
                ShopPane::Shop => MatchPlugin::pane_shop(ui, world)?,
                ShopPane::Info => MatchPlugin::pane_info(ui, world)?,
                ShopPane::Roster => MatchPlugin::pane_roster(ui, world)?,
                ShopPane::Team => MatchPlugin::pane_team(ui, world)?,
                ShopPane::Fusion => MatchPlugin::pane_fusion(ui, world)?,
            },
            Pane::Battle(pane) => match pane {
                BattlePane::View => BattlePlugin::pane_view(ui, world)?,
                BattlePane::EditLeftGraph => BattlePlugin::pane_edit_graph(true, ui, world),
                BattlePane::EditRightGraph => BattlePlugin::pane_edit_graph(false, ui, world),
                BattlePane::BattleEditor => BattleEditorPlugin::pane(ui, world)?,
            },
            Pane::Explorer(pane) => match pane {
                ExplorerPane::Units => ExplorerPlugin::pane_units_list(ui, world)?,
                ExplorerPane::UnitsHouses => ExplorerPlugin::pane_houses(ui, world)?,
                ExplorerPane::Representations => ExplorerPlugin::pane_representations(ui, world)?,
                ExplorerPane::Descriptions => ExplorerPlugin::pane_descriptions(ui, world)?,
                ExplorerPane::Behaviors => ExplorerPlugin::pane_behaviors(ui, world)?,
                ExplorerPane::Stats => ExplorerPlugin::pane_stats(ui, world)?,
                ExplorerPane::Houses => ExplorerPlugin::pane_houses_list(ui, world)?,
                ExplorerPane::HousesUnits => ExplorerPlugin::pane_house_units(ui, world)?,
                ExplorerPane::HouseColors => ExplorerPlugin::pane_house_colors(ui, world)?,
                ExplorerPane::AbilityMagic => ExplorerPlugin::pane_ability_magic(ui, world)?,
                ExplorerPane::StatusMagic => ExplorerPlugin::pane_status_magic(ui, world)?,
            },
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
