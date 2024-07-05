use egui::{Sense, Window};

use super::*;

pub struct UnitContainer {
    faction: Faction,
    slots: usize,
    max_slots: usize,
    left_to_right: bool,
    offset: egui::Vec2,
    top_content: Option<Box<dyn FnOnce(&mut Ui, &mut World) + Send + Sync>>,
    slot_content: Option<Box<dyn Fn(usize, &mut Ui, &mut World) + Send + Sync>>,
    direction: Side,
}

#[derive(Debug)]
pub struct UnitContainerData {
    pub positions: Vec<Pos2>,
}

impl Default for UnitContainerData {
    fn default() -> Self {
        Self {
            positions: vec![pos2(0.0, 0.0)],
        }
    }
}

impl UnitContainer {
    pub fn new(faction: Faction) -> Self {
        Self {
            faction,
            slots: 5,
            max_slots: 5,
            offset: default(),
            left_to_right: false,
            top_content: default(),
            slot_content: default(),
            direction: Side::Top,
        }
    }
    pub fn slots(mut self, value: usize) -> Self {
        self.slots = value;
        self.max_slots = value;
        self
    }
    pub fn max_slots(mut self, value: usize) -> Self {
        self.max_slots = value;
        self
    }
    pub fn left_to_right(mut self) -> Self {
        self.left_to_right = true;
        self
    }
    pub fn offset(mut self, value: [f32; 2]) -> Self {
        self.offset = value.into();
        self
    }
    pub fn direction(mut self, side: Side) -> Self {
        self.direction = side;
        self
    }
    pub fn top_content(
        mut self,
        content: impl FnOnce(&mut Ui, &mut World) + Send + Sync + 'static,
    ) -> Self {
        self.top_content = Some(Box::new(content));
        self
    }
    pub fn slot_content(
        mut self,
        content: impl Fn(usize, &mut Ui, &mut World) + Send + Sync + 'static,
    ) -> Self {
        self.slot_content = Some(Box::new(content));
        self
    }
    pub fn ui(self, data: &mut WidgetData, ui: &mut Ui, world: &mut World) {
        let data = data.unit_container.entry(self.faction).or_insert(default());
        data.positions.resize(self.slots + 1, default());
        let name = format!("{}", self.faction);
        let center = ui.max_rect().center();
        let (anchor, offset) = match self.direction {
            Side::Right => (Align2::LEFT_CENTER, egui::vec2(center.x, 0.0)),
            Side::Left => (Align2::RIGHT_CENTER, egui::vec2(-center.x, 0.0)),
            Side::Top => (Align2::CENTER_BOTTOM, egui::vec2(0.0, -center.y)),
            Side::Bottom => (Align2::CENTER_TOP, egui::vec2(0.0, center.y)),
        };
        const MARGIN: Margin = Margin::same(8.0);
        let max_size = ui.available_rect_before_wrap().width() / self.slots as f32
            - MARGIN.left
            - MARGIN.right;
        const FRAME: Frame = Frame {
            inner_margin: MARGIN,
            outer_margin: Margin::ZERO,
            rounding: Rounding::same(13.0),
            shadow: Shadow::NONE,
            fill: TRANSPARENT,
            stroke: Stroke {
                width: 1.0,
                color: LIGHT_GRAY,
            },
        };
        let resp = Window::new(&name)
            .anchor(anchor, self.offset + offset)
            .constrain_to(ui.available_rect_before_wrap())
            .resizable([true, false])
            .frame(FRAME)
            .title_bar(false)
            .show(ui.ctx(), |ui| {
                if let Some(content) = self.top_content {
                    content(ui, world);
                }
                ui.columns(self.slots, |ui| {
                    for i in 1..=self.slots {
                        let col = if self.left_to_right {
                            i
                        } else {
                            self.slots - i
                        };
                        ui[col].vertical_centered(|ui| {
                            show_frame(i, max_size, i > self.max_slots, data, ui);
                        });
                        if let Some(content) = &self.slot_content {
                            ui[col].vertical_centered_justified(|ui| {
                                content(i, ui, world);
                            });
                        }
                    }
                });
            });
        let rect = resp.unwrap().response.rect;
        {
            let pos = rect.left_top();
            let rect = Rect::from_two_pos(pos, pos + egui::vec2(30.0, -10.0));
            let ui = &mut ui.child_ui(rect, Layout::bottom_up(Align::Min));
            name.cstr().label(ui);
        }
    }
}

fn show_frame(
    ind: usize,
    max_size: f32,
    overflow: bool,
    data: &mut UnitContainerData,
    ui: &mut Ui,
) {
    let size = max_size.min(130.0);
    let (rect, response) = ui.allocate_exact_size(egui::vec2(size, size), Sense::hover());
    data.positions[ind] = rect.center();
    let color = if response.hovered() {
        YELLOW
    } else {
        if overflow {
            RED
        } else {
            LIGHT_GRAY
        }
    };
    let stroke = Stroke { width: 1.0, color };
    const DASH: f32 = 10.0;
    const GAP: f32 = 20.0;
    let ind_rect = Rect::from_min_max(
        rect.right_top() + egui::vec2(-10.0, 5.0),
        rect.right_top() + egui::vec2(-5.0, 0.0),
    );
    {
        let ui = &mut ui.child_ui(ind_rect, Layout::top_down(Align::Max));
        ind.to_string().cstr_cs(color, CstrStyle::Bold).label(ui);
    }
    ui.painter().add(egui::Shape::dashed_line(
        &[rect.left_top(), rect.right_top()],
        stroke,
        DASH,
        GAP,
    ));
    ui.painter().add(egui::Shape::dashed_line(
        &[rect.right_top(), rect.right_bottom()],
        stroke,
        DASH,
        GAP,
    ));
    ui.painter().add(egui::Shape::dashed_line(
        &[rect.right_bottom(), rect.left_bottom()],
        stroke,
        DASH,
        GAP,
    ));
    ui.painter().add(egui::Shape::dashed_line(
        &[rect.left_bottom(), rect.left_top()],
        stroke,
        DASH,
        GAP,
    ));
}
