use egui::{Sense, Window};

use super::*;

pub struct UnitContainer {
    faction: Faction,
    slots: usize,
    left_to_right: bool,
}

#[derive(Default)]
pub struct UnitContainerData {
    pub positions: Vec<Pos2>,
}

impl UnitContainer {
    pub fn new(faction: Faction) -> Self {
        Self {
            faction,
            slots: 5,
            left_to_right: false,
        }
    }
    pub fn slots(mut self, value: usize) -> Self {
        self.slots = value;
        self
    }
    pub fn left_to_right(mut self) -> Self {
        self.left_to_right = true;
        self
    }
    pub fn ui(self, data: &mut WidgetData, ui: &mut Ui) {
        let data = data.unit_container.entry(self.faction).or_insert(default());
        data.positions.resize(self.slots, default());
        Window::new("Unit Container")
            .anchor(Align2::CENTER_CENTER, [0.0, -150.0])
            .resizable([true, false])
            .frame(Frame {
                inner_margin: Margin::same(8.0),
                outer_margin: default(),
                rounding: Rounding::same(13.0),
                shadow: default(),
                fill: TRANSPARENT,
                stroke: Stroke {
                    width: 1.0,
                    color: LIGHT_GRAY,
                },
            })
            .title_bar(false)
            .show(ui.ctx(), |ui| {
                ui.columns(self.slots, |ui| {
                    for i in 0..self.slots {
                        let col = if self.left_to_right {
                            i
                        } else {
                            self.slots - i - 1
                        };
                        ui[col].vertical_centered(|ui| {
                            show_frame(i, data, ui);
                            Button::click("Test").ui(ui);
                            if i == 1 {
                                Button::click("Test 1").ui(ui);
                                Button::click("Test 2").ui(ui);
                            }
                        });
                    }
                });
            });
    }
}

fn show_frame(ind: usize, data: &mut UnitContainerData, ui: &mut Ui) {
    const SIZE: f32 = 130.0;
    let (rect, response) = ui.allocate_exact_size(egui::vec2(SIZE, SIZE), Sense::hover());
    data.positions[ind] = rect.center();
    let color = if response.hovered() {
        YELLOW
    } else {
        LIGHT_GRAY
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
        (ind + 1).to_string().cstr_c(color).label(ui);
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
