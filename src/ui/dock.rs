use egui::{CentralPanel, CornerRadius, Frame, TopBottomPanel, Ui, WidgetText};
use egui_dock::{DockArea, DockState, Style, TabViewer};

use super::*;

pub struct DockTree {
    pub state: DockState<Tab>,
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

#[derive(Default)]
struct DockContext {
    style: Option<Style>,
    world: Option<World>,
}

impl TabViewer for DockContext {
    type Tab = Tab;

    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        match tab {
            Tab::IncubatorHouse => "House".into(),
            Tab::IncubatorHouseColor => "House Color".into(),
            Tab::IncubatorActionAbility => "Action Ability".into(),
            Tab::IncubatorUnit => "Unit".into(),
            Tab::IncubatorUnitStats => "Unit Stats".into(),
            Tab::IncubatorUnitDescription => "Unit Description".into(),
            Tab::IncubatorRepresentation => "Representation".into(),
            Tab::IncubatorNewNode => "New Node".into(),
            _ => tab.as_ref().to_case(Case::Title).into(),
        }
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        let world = self.world.as_mut().unwrap();
        match tab.ui(ui, world) {
            Ok(_) => {}
            Err(e) => format!("Tab {tab} error: {e}").notify_error(world),
        }
    }

    fn closeable(&mut self, tab: &mut Self::Tab) -> bool {
        false
    }

    fn on_close(&mut self, tab: &mut Self::Tab) -> bool {
        true
    }
}

impl Into<Vec<Tab>> for Tab {
    fn into(self) -> Vec<Tab> {
        [self].into()
    }
}

impl DockTree {
    pub fn new(root: Tab) -> Self {
        let state = DockState::new([root].into());

        let context = DockContext {
            style: None,
            world: None,
        };

        Self { context, state }
    }
    pub fn add_tab(&mut self, tab: Tab) {
        self.state.main_surface_mut().push_to_focused_leaf(tab);
    }
    pub fn ui(&mut self, ctx: &egui::Context, world: World) -> World {
        self.context.world = Some(world);
        TopBottomPanel::top("egui_dock::MenuBar").show(ctx, |ui| egui::menu::bar(ui, |ui| {}));
        CentralPanel::default()
            .frame(Frame::central_panel(&ctx.style()).inner_margin(8.))
            .show(ctx, |ui| {
                let style = self
                    .context
                    .style
                    .get_or_insert_with(|| {
                        let mut style = Style::from_egui(ui.style());
                        style.separator.color_idle = TRANSPARENT;
                        style.separator.color_hovered = TRANSPARENT;
                        style.separator.width = 8.0;
                        style.tab.tab_body.inner_margin = 13.into();
                        style.tab.tab_body.corner_radius = CornerRadius {
                            nw: 0,
                            ne: 0,
                            sw: 13,
                            se: 13,
                        };
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
