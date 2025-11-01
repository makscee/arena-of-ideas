use bevy_egui::egui::scroll_area::ScrollBarVisibility::AlwaysHidden;
use egui_tiles::SimplificationOptions;

use super::*;

#[derive(Default)]
pub struct TreeBehavior {
    pub world: Option<World>,
    pub tile_names: HashMap<TileId, String>,
}

static CURRENT_TILE_ID: Mutex<u64> = Mutex::new(0);
pub fn cur_tile_id() -> TileId {
    TileId::from_u64(*CURRENT_TILE_ID.lock())
}
impl egui_tiles::Behavior<Pane> for TreeBehavior {
    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        tile_id: TileId,
        view: &mut Pane,
    ) -> egui_tiles::UiResponse {
        *CURRENT_TILE_ID.lock() = tile_id.0;
        dark_frame().show(ui, |ui| {
            ScrollArea::both()
                .scroll_bar_visibility(AlwaysHidden)
                .show(ui, |ui| {
                    ui.expand_to_include_rect(ui.available_rect_before_wrap());
                    if let Some(world) = self.world.as_mut() {
                        if let Err(e) = view.ui(ui, world) {
                            e.ui(ui);
                        }
                    }
                });
        });
        egui_tiles::UiResponse::None
    }
    fn simplification_options(&self) -> SimplificationOptions {
        SimplificationOptions {
            all_panes_must_have_tabs: true,
            ..default()
        }
    }
    fn tab_text_color(
        &self,
        visuals: &egui::Visuals,
        _tiles: &Tiles<Pane>,
        _tile_id: TileId,
        state: &egui_tiles::TabState,
    ) -> Color32 {
        if state.active {
            high_contrast_text()
        } else if state.is_being_dragged {
            YELLOW
        } else {
            visuals.weak_text_color()
        }
    }
    fn tab_bar_color(&self, _: &egui::Visuals) -> Color32 {
        TRANSPARENT
    }
    fn tab_bg_color(
        &self,
        _: &egui::Visuals,
        _: &Tiles<Pane>,
        _: TileId,
        _: &egui_tiles::TabState,
    ) -> Color32 {
        TRANSPARENT
    }
    fn tab_outline_stroke(
        &self,
        _visuals: &egui::Visuals,
        _tiles: &Tiles<Pane>,
        _tile_id: TileId,
        _state: &egui_tiles::TabState,
    ) -> Stroke {
        Stroke::NONE
    }
    fn resize_stroke(&self, _: &egui::Style, resize_state: egui_tiles::ResizeState) -> Stroke {
        match resize_state {
            egui_tiles::ResizeState::Idle => Stroke::NONE,
            egui_tiles::ResizeState::Hovering => {
                Stroke::new(1.0, ui_element_border_and_focus_rings())
            }
            egui_tiles::ResizeState::Dragging => Stroke::new(1.0, hovered_ui_element_border()),
        }
    }
    fn on_edit(&mut self, action: egui_tiles::EditAction) {
        match action {
            egui_tiles::EditAction::TileResized
            | egui_tiles::EditAction::TileDropped
            | egui_tiles::EditAction::TabSelected => {
                if let Some(world) = self.world.as_mut() {
                    TilePlugin::request_tree_save(cur_state(world));
                }
            }
            _ => {}
        }
    }

    fn tab_title_for_pane(&mut self, view: &Pane) -> egui::WidgetText {
        match view {
            Pane::Explorer(pane) => pane.to_string().into(),
            Pane::Battle(pane) => match pane {
                BattlePane::View => "Battle View".into(),
                BattlePane::EditLeftGraph => "Left Team".into(),
                BattlePane::EditRightGraph => "Right Team".into(),
                BattlePane::TeamEditor(is_left) => {
                    if *is_left {
                        "Left Team Editor".into()
                    } else {
                        "Right Team Editor".into()
                    }
                }
            },
            Pane::Shop(pane) => pane.as_ref().into(),
            _ => view.as_ref().into(),
        }
    }

    fn is_tab_closable(&self, _tiles: &Tiles<Pane>, _tile_id: TileId) -> bool {
        false
    }

    fn on_tab_close(&mut self, tiles: &mut Tiles<Pane>, tile_id: TileId) -> bool {
        if let Some(tile) = tiles.get(tile_id) {
            match tile {
                Tile::Pane(pane) => {
                    let tab_title = self.tab_title_for_pane(pane);
                    log::debug!("Closing tab: {}, tile ID: {tile_id:?}", tab_title.text());
                }
                Tile::Container(container) => {
                    log::debug!("Closing container: {:?}", container.kind());
                    let children_ids = container.children();
                    for child_id in children_ids {
                        if let Some(Tile::Pane(pane)) = tiles.get(*child_id) {
                            let tab_title = self.tab_title_for_pane(pane);
                            log::debug!("Closing tab: {}, tile ID: {tile_id:?}", tab_title.text());
                        }
                    }
                }
            }
        }
        true
    }
    fn tab_title_for_tile(&mut self, tiles: &Tiles<Pane>, tile_id: TileId) -> WidgetText {
        if let Some(name) = self.tile_names.get(&tile_id) {
            return name.into();
        }
        if let Some(tile) = tiles.get(tile_id) {
            match tile {
                Tile::Pane(pane) => self.tab_title_for_pane(pane),
                Tile::Container(container) => format!("{:?}", container.kind()).into(),
            }
        } else {
            "MISSING TILE".into()
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct TileTree {
    pub tree: Tree<Pane>,
    #[serde(skip)]
    pub behavior: TreeBehavior,
}

impl Default for TileTree {
    fn default() -> Self {
        Self {
            tree: Tree::new_horizontal("tree", default()),
            behavior: Default::default(),
        }
    }
}

impl TileTree {
    pub fn show(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Apply global colorix background
            self.tree.ui(&mut self.behavior, ui);
        });
    }
}

pub trait TreeExt {
    fn add_tab(&mut self, tab: TileId, new: TileId) -> NodeResult<()>;
}

impl TreeExt for Tree<Pane> {
    fn add_tab(&mut self, cur: TileId, new: TileId) -> NodeResult<()> {
        let cur_tile = self
            .tiles
            .get_mut(cur)
            .to_custom_e("Failed to get current tile")?;
        match cur_tile {
            Tile::Pane(_) => {
                let container = self
                    .tiles
                    .parent_of(cur)
                    .to_custom_e("Failed to get parent of current tile")?;
                match self.tiles.get_mut(container).unwrap() {
                    Tile::Pane(_) => unreachable!(),
                    Tile::Container(container) => container.add_child(new),
                }
            }
            Tile::Container(container) => {
                container.add_child(new);
            }
        }
        self.make_active(|tile_id, _| tile_id == new);
        Ok(())
    }
}

pub trait TileIdExt {
    fn to_tab(self, tiles: &mut Tiles<Pane>) -> Self;
    fn with_name(self, tile_tree: &mut TileTree, name: impl ToString) -> Self;
}

impl TileIdExt for TileId {
    fn to_tab(self, tiles: &mut Tiles<Pane>) -> TileId {
        tiles.insert_tab_tile([self].into())
    }

    fn with_name(self, tile_tree: &mut TileTree, name: impl ToString) -> Self {
        tile_tree.behavior.tile_names.insert(self, name.to_string());
        self
    }
}
