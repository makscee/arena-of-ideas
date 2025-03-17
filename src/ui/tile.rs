use egui_tiles::SimplificationOptions;

use super::*;

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

    Admin,

    WorldInspector,
}

impl Into<Vec<Pane>> for Pane {
    fn into(self) -> Vec<Pane> {
        [self].into()
    }
}

impl Pane {
    pub fn ui(self, ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        match self {
            Self::MainMenu => {
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
            Self::Login => LoginPlugin::tab_login(ui, world),
            Self::Register => LoginPlugin::tab_register(ui, world),
            Self::Connect => ConnectPlugin::tab(ui),
            Self::Admin => AdminPlugin::tab(ui, world),
            Self::Shop => MatchPlugin::tab_shop(ui, world)?,
            Self::Roster => match cur_state(world) {
                GameState::Match => MatchPlugin::tab_roster(ui, world)?,
                GameState::FusionEditor => FusionEditorPlugin::roster_tab(ui, world)?,
                _ => unreachable!(),
            },
            Self::Team => MatchPlugin::tab_team(ui, world)?,
            Self::Triggers => FusionEditorPlugin::tab_triggers(ui, world),
            Self::Actions => FusionEditorPlugin::tab_actions(ui, world),
            Self::FusionResult => FusionEditorPlugin::tab_fusion_result(ui, world)?,
            Self::BattleEditor => BattleEditorPlugin::tab(ui, world)?,

            Self::IncubatorNewNode => IncubatorPlugin::tab_new_node(ui, world)?,
            Self::IncubatorInspect => IncubatorPlugin::tab_inspect(ui, world)?,
            Self::IncubatorNodes => IncubatorPlugin::tab_nodes(ui, world)?,

            Self::WorldInspector => bevy_inspector_egui::bevy_inspector::ui_for_world(world, ui),
        };
        Ok(())
    }
}

#[derive(Default)]
pub struct TreeBehavior {
    pub world: Option<World>,
}

impl egui_tiles::Behavior<Pane> for TreeBehavior {
    fn simplification_options(&self) -> SimplificationOptions {
        SimplificationOptions {
            all_panes_must_have_tabs: true,
            ..default()
        }
    }
    fn tab_text_color(
        &self,
        _visuals: &egui::Visuals,
        _tiles: &Tiles<Pane>,
        _tile_id: TileId,
        state: &egui_tiles::TabState,
    ) -> Color32 {
        if state.active {
            VISIBLE_DARK
        } else if state.is_being_dragged {
            YELLOW
        } else {
            BG_LIGHT
        }
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
            egui_tiles::ResizeState::Hovering => STROKE_LIGHT,
            egui_tiles::ResizeState::Dragging => STROKE_YELLOW,
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
    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        _tile_id: egui_tiles::TileId,
        view: &mut Pane,
    ) -> egui_tiles::UiResponse {
        ScrollArea::both().show(ui, |ui| {
            DARK_FRAME.show(ui, |ui| {
                if let Some(world) = self.world.as_mut() {
                    view.ui(ui, world).log();
                }
                ui.expand_to_include_rect(ui.available_rect_before_wrap());
            });
        });
        egui_tiles::UiResponse::None
    }

    fn tab_title_for_pane(&mut self, view: &Pane) -> egui::WidgetText {
        view.to_string().into()
    }

    fn is_tab_closable(&self, _tiles: &Tiles<Pane>, _tile_id: TileId) -> bool {
        true
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
            self.tree.ui(&mut self.behavior, ui);
        });
    }
}
