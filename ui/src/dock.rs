use egui::{CentralPanel, ComboBox, CornerRadius, Frame, Slider, TopBottomPanel, Ui, WidgetText};

use egui_dock::{
    AllowedSplits, DockArea, DockState, NodeIndex, OverlayType, Style, SurfaceIndex,
    TabInteractionStyle, TabViewer,
};
use strum_macros::AsRefStr;

use super::*;

pub struct DockTree {
    pub state: DockState<TabContent>,
    context: DockContext,
}

impl Default for DockTree {
    fn default() -> Self {
        Self {
            state: DockState::new(default()),
            context: default(),
        }
    }
}

pub struct TabContent {
    pub name: String,
    content: Box<dyn FnMut(&mut Ui, &mut World) + Send + Sync>,
}

#[derive(PartialEq, Eq, Clone, Copy, Hash, AsRefStr)]
pub enum TabType {
    MainMenu,
    Shop,
}

#[derive(Default)]
struct DockContext {
    style: Option<Style>,
    world: Option<World>,
}

impl TabViewer for DockContext {
    type Tab = TabContent;

    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        tab.name.as_str().into()
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        let world = self.world.as_mut().unwrap();
        (tab.content)(ui, world);
    }

    fn closeable(&mut self, tab: &mut Self::Tab) -> bool {
        false
    }

    fn on_close(&mut self, tab: &mut Self::Tab) -> bool {
        true
    }
}

impl TabContent {
    pub fn new(
        name: impl ToString,
        content: impl FnMut(&mut Ui, &mut World) + Send + Sync + 'static,
    ) -> Self {
        Self {
            name: name.to_string(),
            content: Box::new(content),
        }
    }
    pub fn new_vec(
        name: impl ToString,
        content: impl FnMut(&mut Ui, &mut World) + Send + Sync + 'static,
    ) -> Vec<Self> {
        [Self::new(name, content)].into()
    }
}

impl DockTree {
    pub fn new(root: TabContent) -> Self {
        let state = DockState::new([root].into());

        let context = DockContext {
            style: None,
            world: None,
        };

        Self { context, state }
    }
    pub fn add_tab(&mut self, tab: TabContent) {
        self.state.main_surface_mut().push_to_focused_leaf(tab);
    }
    pub fn ui(&mut self, ctx: &egui::Context, world: World) -> World {
        self.context.world = Some(world);
        TopBottomPanel::top("egui_dock::MenuBar").show(ctx, |ui| egui::menu::bar(ui, |ui| {}));
        CentralPanel::default()
            .frame(Frame::central_panel(&ctx.style()).inner_margin(0.))
            .show(ctx, |ui| {
                let style = self
                    .context
                    .style
                    .get_or_insert_with(|| {
                        let mut style = Style::from_egui(ui.style());
                        style.dock_area_padding = Some(Margin::same(8));
                        style.separator.color_idle = TRANSPARENT;
                        style.separator.color_hovered = TRANSPARENT;
                        style.separator.width = 8.0;
                        style.tab.tab_body.bg_fill = BG_DARK;
                        style.tab.tab_body.inner_margin = 13.into();
                        style.tab.tab_body.corner_radius = CornerRadius {
                            nw: 0,
                            ne: 0,
                            sw: 13,
                            se: 13,
                        };
                        style.tab.active.bg_fill = BG_DARK;
                        style.tab.focused.bg_fill = BG_DARK;
                        style.tab_bar.fill_tab_bar = true;
                        style
                    })
                    .clone();
                DockArea::new(&mut self.state)
                    .style(style)
                    .show_leaf_close_all_buttons(false)
                    .show_leaf_collapse_buttons(false)
                    .show_inside(ui, &mut self.context);
            });
        self.context.world.take().unwrap()
    }
}
