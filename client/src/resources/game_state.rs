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

                // Left team tab with editor and graph
                let left_team_editor =
                    tiles.insert_pane(Pane::Battle(BattlePane::TeamEditor(true)));
                let left_team_graph = tiles.insert_pane(Pane::Battle(BattlePane::EditLeftGraph));
                let left_team_content =
                    tiles.insert_horizontal_tile([left_team_editor, left_team_graph].into());
                if let Tile::Container(h) = tiles.get_mut(left_team_content).unwrap() {
                    if let Container::Linear(h) = h {
                        h.shares.set_share(left_team_editor, 2.0);
                        h.shares.set_share(left_team_graph, 1.0);
                    }
                }
                let left_team_tab = left_team_content.with_name(tile_tree, "Left Team");

                // Right team tab with editor and graph
                let right_team_editor =
                    tiles.insert_pane(Pane::Battle(BattlePane::TeamEditor(false)));
                let right_team_graph = tiles.insert_pane(Pane::Battle(BattlePane::EditRightGraph));
                let right_team_content =
                    tiles.insert_horizontal_tile([right_team_editor, right_team_graph].into());
                if let Tile::Container(h) = tiles.get_mut(right_team_content).unwrap() {
                    if let Container::Linear(h) = h {
                        h.shares.set_share(right_team_editor, 2.0);
                        h.shares.set_share(right_team_graph, 1.0);
                    }
                }
                let right_team_tab = right_team_content.with_name(tile_tree, "Right Team");

                let team_tabs = tiles.insert_tab_tile([left_team_tab, right_team_tab].into());
                let root = tiles.insert_vertical_tile([view, team_tabs].into());
                if let Tile::Container(v) = tiles.get_mut(root).unwrap() {
                    if let Container::Linear(v) = v {
                        v.shares.set_share(view, 1.0);
                        v.shares.set_share(team_tabs, 2.0);
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

                let unit_description =
                    tiles.insert_pane(Pane::Explorer(ExplorerPane::UnitDescription));
                let unit_behavior = tiles.insert_pane(Pane::Explorer(ExplorerPane::UnitBehavior));
                let unit_representation =
                    tiles.insert_pane(Pane::Explorer(ExplorerPane::UnitRepresentation));
                let unit_stats = tiles.insert_pane(Pane::Explorer(ExplorerPane::UnitStats));

                let middle_top =
                    tiles.insert_horizontal_tile([unit_description, unit_behavior].into());
                let middle_bottom =
                    tiles.insert_horizontal_tile([unit_representation, unit_stats].into());
                let middle_grid = tiles.insert_vertical_tile([middle_top, middle_bottom].into());

                let unit_parent_list =
                    tiles.insert_pane(Pane::Explorer(ExplorerPane::UnitParentList));

                let units_content = tiles
                    .insert_horizontal_tile([left_column, middle_grid, unit_parent_list].into());
                let units_tab = units_content.with_name(tile_tree, "Units");

                // Create Houses tab layout
                let houses_list = tiles.insert_pane(Pane::Explorer(ExplorerPane::HousesList));
                let house_card = tiles.insert_pane(Pane::Explorer(ExplorerPane::HouseCard));
                let houses_left_column =
                    tiles.insert_vertical_tile([houses_list, house_card].into());

                let house_abilities_list =
                    tiles.insert_pane(Pane::Explorer(ExplorerPane::HouseAbilitiesList));
                let house_statuses_list =
                    tiles.insert_pane(Pane::Explorer(ExplorerPane::HouseStatusesList));
                let house_units_list =
                    tiles.insert_pane(Pane::Explorer(ExplorerPane::HouseUnitsList));

                let houses_content = tiles.insert_horizontal_tile(
                    [
                        houses_left_column,
                        house_abilities_list,
                        house_statuses_list,
                        house_units_list,
                    ]
                    .into(),
                );
                let houses_tab = houses_content.with_name(tile_tree, "Houses");

                // Create Abilities tab layout
                let abilities_list = tiles.insert_pane(Pane::Explorer(ExplorerPane::AbilitiesList));
                let ability_card = tiles.insert_pane(Pane::Explorer(ExplorerPane::AbilityCard));
                let abilities_left_column =
                    tiles.insert_vertical_tile([abilities_list, ability_card].into());

                let ability_description =
                    tiles.insert_pane(Pane::Explorer(ExplorerPane::AbilityDescription));
                let ability_effect = tiles.insert_pane(Pane::Explorer(ExplorerPane::AbilityEffect));
                let abilities_middle =
                    tiles.insert_vertical_tile([ability_description, ability_effect].into());

                let ability_parent_list =
                    tiles.insert_pane(Pane::Explorer(ExplorerPane::AbilityParentList));

                let abilities_content = tiles.insert_horizontal_tile(
                    [abilities_left_column, abilities_middle, ability_parent_list].into(),
                );
                let abilities_tab = abilities_content.with_name(tile_tree, "Abilities");

                // Create Statuses tab layout
                let statuses_list = tiles.insert_pane(Pane::Explorer(ExplorerPane::StatusesList));
                let status_card = tiles.insert_pane(Pane::Explorer(ExplorerPane::StatusCard));
                let statuses_left_column =
                    tiles.insert_vertical_tile([statuses_list, status_card].into());

                let status_description =
                    tiles.insert_pane(Pane::Explorer(ExplorerPane::StatusDescription));
                let status_behavior =
                    tiles.insert_pane(Pane::Explorer(ExplorerPane::StatusBehavior));
                let status_representation =
                    tiles.insert_pane(Pane::Explorer(ExplorerPane::StatusRepresentation));
                let statuses_middle = tiles.insert_vertical_tile(
                    [status_description, status_behavior, status_representation].into(),
                );

                let status_parent_list =
                    tiles.insert_pane(Pane::Explorer(ExplorerPane::StatusParentList));

                let statuses_content = tiles.insert_horizontal_tile(
                    [statuses_left_column, statuses_middle, status_parent_list].into(),
                );
                let statuses_tab = statuses_content.with_name(tile_tree, "Statuses");

                // Main tabs
                let root = tiles
                    .insert_tab_tile([units_tab, houses_tab, abilities_tab, statuses_tab].into());
                tile_tree.tree = Tree::new(TREE_ID, root, tiles);
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
    Admin,
}

#[derive(PartialEq, Eq, Clone, Copy, Hash, AsRefStr, Serialize, Deserialize, Debug, Display)]
pub enum BattlePane {
    View,
    EditLeftGraph,
    EditRightGraph,
    TeamEditor(bool), // true for left, false for right
}
#[derive(PartialEq, Eq, Clone, Copy, Hash, AsRefStr, Serialize, Deserialize, Debug, Display)]
pub enum ShopPane {
    Shop,
    Roster,
    Team,
    Info,
    Fusion,
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
                BattlePane::TeamEditor(is_left) => TeamEditorPlugin::pane(is_left, world, ui),
            },
            Pane::Explorer(pane) => ExplorerPlugin::pane(pane, ui, world)?,
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
