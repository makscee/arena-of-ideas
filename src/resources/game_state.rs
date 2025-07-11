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
                let controls = tiles.insert_pane(Pane::Battle(BattlePane::Controls));
                let edit_left_graph = tiles.insert_pane(Pane::Battle(BattlePane::EditLeftGraph));
                let edit_left_slots = tiles.insert_pane(Pane::Battle(BattlePane::EditLeftSlots));
                let edit_right_graph = tiles.insert_pane(Pane::Battle(BattlePane::EditRightGraph));
                let edit_right_slots = tiles.insert_pane(Pane::Battle(BattlePane::EditRightSlots));
                let edit_left =
                    tiles.insert_vertical_tile([edit_left_slots, edit_left_graph].into());
                let edit_right =
                    tiles.insert_vertical_tile([edit_right_slots, edit_right_graph].into());
                let edit = tiles.insert_tab_tile([edit_left, edit_right].into());
                let vertical = tiles.insert_vertical_tile([view, controls].into());
                let root = tiles.insert_horizontal_tile([edit, vertical].into());
                tile_tree.tree = Tree::new(TREE_ID, root, tiles);
            }
            GameState::Explorer => {
                let mut tiles = Tiles::default();

                let categories = NodeKindCategory::iter()
                    .map(|c| {
                        let kinds = c
                            .kinds()
                            .into_iter()
                            .filter_map(|k| {
                                if k == NodeKind::None {
                                    None
                                } else {
                                    Some(tiles.insert_pane(Pane::ExplorerList(k)))
                                }
                            })
                            .collect_vec();

                        let tile = match kinds.len() {
                            0 => tiles.insert_pane(Pane::ExplorerList(NodeKind::None)),
                            1 => kinds[0],
                            2 => tiles.insert_horizontal_tile(kinds),
                            3 => tiles.insert_horizontal_tile(kinds),
                            4 => {
                                let top = tiles.insert_horizontal_tile([kinds[0], kinds[1]].into());
                                let bottom =
                                    tiles.insert_horizontal_tile([kinds[2], kinds[3]].into());
                                tiles.insert_vertical_tile([top, bottom].into())
                            }
                            5 => {
                                let top = tiles.insert_horizontal_tile([kinds[0], kinds[1]].into());
                                let bottom = tiles
                                    .insert_horizontal_tile([kinds[2], kinds[3], kinds[4]].into());
                                tiles.insert_vertical_tile([top, bottom].into())
                            }
                            6 => {
                                let top = tiles
                                    .insert_horizontal_tile([kinds[0], kinds[1], kinds[2]].into());
                                let bottom = tiles
                                    .insert_horizontal_tile([kinds[3], kinds[4], kinds[5]].into());
                                tiles.insert_vertical_tile([top, bottom].into())
                            }
                            _ => {
                                // For 7 or more, arrange in rows of max 3 columns
                                let mut rows = Vec::new();
                                for chunk in kinds.chunks(3) {
                                    let row = tiles.insert_horizontal_tile(chunk.to_vec());
                                    rows.push(row);
                                }
                                tiles.insert_vertical_tile(rows)
                            }
                        };

                        tile_tree.behavior.tile_names.insert(tile, c.to_string());
                        tile
                    })
                    .collect_vec();
                let root = tiles.insert_tab_tile(categories);
                tile_tree.tree = Tree::new(TREE_ID, root, tiles);
            }
            GameState::Shop => {
                let mut tiles = Tiles::default();
                let shop = tiles.insert_pane(Pane::Shop(ShopPane::Shop));
                let info = tiles.insert_pane(Pane::Shop(ShopPane::Info));
                let roster = tiles.insert_pane(Pane::Shop(ShopPane::Roster));
                let team = tiles.insert_pane(Pane::Shop(ShopPane::Team));
                let fusion = tiles.insert_pane(Pane::Shop(ShopPane::Fusion));
                let top = tiles.insert_horizontal_tile([shop, info].into());
                if let Tile::Container(h) = tiles.get_mut(top).unwrap() {
                    if let Container::Linear(h) = h {
                        h.shares.set_share(shop, 4.0);
                    }
                }
                let mid = tiles.insert_horizontal_tile([team, roster, fusion].into());
                if let Tile::Container(h) = tiles.get_mut(mid).unwrap() {
                    if let Container::Linear(h) = h {
                        h.shares.set_share(team, 4.0);
                    }
                }
                let root = tiles.insert_vertical_tile([top, fusion].into());
                tile_tree.tree = Tree::new(TREE_ID, root, tiles);
            }
            GameState::Battle => {
                tile_tree.tree = Tree::new_vertical(
                    TREE_ID,
                    [
                        Pane::Battle(BattlePane::View),
                        Pane::Battle(BattlePane::Controls),
                    ]
                    .into(),
                );
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
    ExplorerList(NodeKind),
    ExplorerInspect(ExplorerPane),
}

#[derive(PartialEq, Eq, Clone, Copy, Hash, AsRefStr, Serialize, Deserialize, Debug, Display)]
pub enum BattlePane {
    View,
    Controls,
    EditLeftGraph,
    EditLeftSlots,
    EditRightGraph,
    EditRightSlots,
}
#[derive(PartialEq, Eq, Clone, Copy, Hash, AsRefStr, Serialize, Deserialize, Debug, Display)]
pub enum ShopPane {
    Shop,
    Roster,
    Hand,
    Team,
    Info,
    Fusion,
}
#[derive(PartialEq, Eq, Clone, Copy, Hash, AsRefStr, Serialize, Deserialize, Debug, Display)]
pub enum ExplorerPane {
    Selected,
    Parents,
    Children,
    Node,
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
                ShopPane::Hand => MatchPlugin::pane_hand(ui, world)?,
                ShopPane::Team => MatchPlugin::pane_team(ui, world)?,
                ShopPane::Fusion => MatchPlugin::pane_fusion(ui, world)?,
            },
            Pane::Battle(pane) => match pane {
                BattlePane::View => BattlePlugin::pane_view(ui, world)?,
                BattlePane::Controls => BattlePlugin::pane_controls(ui, world)?,
                BattlePane::EditLeftGraph => BattlePlugin::pane_edit_graph(true, ui, world),
                BattlePane::EditRightGraph => BattlePlugin::pane_edit_graph(false, ui, world),
                BattlePane::EditLeftSlots => BattlePlugin::pane_edit_slots(true, ui, world),
                BattlePane::EditRightSlots => BattlePlugin::pane_edit_slots(false, ui, world),
            },
            Pane::ExplorerInspect(pane) => match pane {
                ExplorerPane::Selected => NodeExplorerPlugin::pane_selected(ui, world)?,
                ExplorerPane::Parents => NodeExplorerPlugin::pane_parents(ui, world)?,
                ExplorerPane::Children => NodeExplorerPlugin::pane_children(ui, world)?,
                ExplorerPane::Node => NodeExplorerPlugin::pane_node(ui, world)?,
            },
            Pane::ExplorerList(kind) => {
                kind.cstr_s(CstrStyle::Heading2).label(ui);
                NodeExplorerPluginNew::pane_kind_list(ui, world, kind)?
            }
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
