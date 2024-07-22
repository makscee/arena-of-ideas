use egui::{Sense, Window};

use super::*;

pub struct UnitContainer {
    faction: Faction,
    slots: usize,
    max_slots: usize,
    right_to_left: bool,
    hug_unit: bool,
    top_content: Option<Box<dyn FnOnce(&mut Ui, &mut World) + Send + Sync>>,
    slot_content: Option<Box<dyn Fn(usize, Option<Entity>, &mut Ui, &mut World) + Send + Sync>>,
    hover_content: Option<Box<dyn Fn(usize, Option<Entity>, &mut Ui, &mut World) + Send + Sync>>,
    pivot: Align2,
    position: egui::Vec2,
}

#[derive(Debug)]
pub struct UnitContainerData {
    pub positions: Vec<Pos2>,
    pub entities: Vec<Option<Entity>>,
}

impl Default for UnitContainerData {
    fn default() -> Self {
        Self {
            positions: vec![pos2(0.0, 0.0)],
            entities: vec![None],
        }
    }
}

impl UnitContainer {
    pub fn new(faction: Faction) -> Self {
        Self {
            faction,
            slots: 5,
            max_slots: 5,
            right_to_left: false,
            hug_unit: false,
            top_content: default(),
            slot_content: default(),
            hover_content: default(),
            pivot: Align2::CENTER_CENTER,
            position: default(),
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
    pub fn right_to_left(mut self) -> Self {
        self.right_to_left = true;
        self
    }
    pub fn hug_unit(mut self) -> Self {
        self.hug_unit = true;
        self
    }
    pub fn position(mut self, value: egui::Vec2) -> Self {
        self.position = value;
        self
    }
    pub fn pivot(mut self, value: Align2) -> Self {
        self.pivot = value;
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
        content: impl Fn(usize, Option<Entity>, &mut Ui, &mut World) + Send + Sync + 'static,
    ) -> Self {
        self.slot_content = Some(Box::new(content));
        self
    }
    pub fn hover_content(
        mut self,
        content: impl Fn(usize, Option<Entity>, &mut Ui, &mut World) + Send + Sync + 'static,
    ) -> Self {
        self.hover_content = Some(Box::new(content));
        self
    }
    pub fn ui(self, data: &mut WidgetData, ui: &mut Ui, world: &mut World) {
        let data = data.unit_container.entry(self.faction).or_insert(default());
        data.positions.resize(self.slots + 1, default());
        data.entities.resize(self.slots + 1, None);
        let name = format!("{}", self.faction);
        ui.ctx().add_path(&name);
        const MARGIN: Margin = Margin::same(8.0);
        let available_rect = ui.available_rect_before_wrap();
        let pos = available_rect.min + self.position * available_rect.size();
        let max_size = if self.hug_unit {
            CameraPlugin::pixel_unit(ui.ctx(), world) * 2.0
        } else {
            available_rect.width() / self.slots as f32 - MARGIN.left - MARGIN.right
        };
        const FRAME: Frame = Frame {
            inner_margin: MARGIN,
            outer_margin: MARGIN,
            rounding: Rounding::same(13.0),
            shadow: Shadow::NONE,
            fill: TRANSPARENT,
            stroke: Stroke {
                width: 1.0,
                color: VISIBLE_DARK,
            },
        };
        let mut hovered_rect: Option<(usize, Rect)> = None;
        let resp = Window::new(&name)
            .fixed_pos(pos)
            .pivot(self.pivot)
            .constrain_to(ui.available_rect_before_wrap())
            .resizable([true, false])
            .frame(FRAME)
            .title_bar(false)
            .show(ui.ctx(), |ui| {
                if let Some(content) = self.top_content {
                    content(ui, world);
                }
                ui.columns(self.slots, |ui| {
                    for i in 0..self.slots {
                        let col = if self.right_to_left {
                            i
                        } else {
                            self.slots - i - 1
                        };
                        let ui = &mut ui[col];
                        ui.ctx().add_path(&i.to_string());
                        let entity = data.entities[i];
                        ui.vertical_centered(|ui| {
                            let name = if let Some(entity) = entity {
                                Some(entity_name(entity))
                            } else {
                                None
                            };
                            let response =
                                show_frame(i, max_size, i >= self.max_slots, name, data, ui);
                            if response.hovered() && ui.ctx().dragged_id().is_none() {
                                hovered_rect = Some((i, response.rect));
                            }
                            if response.drag_started() {
                                ui.ctx()
                                    .drag_start(response.interact_pointer_pos().unwrap());
                            }
                            if response.drag_stopped() {
                                ui.ctx().drag_end();
                            }
                        });
                        if let Some(content) = &self.slot_content {
                            ui.vertical_centered_justified(|ui| {
                                content(i, entity, ui, world);
                            });
                        }
                        ui.ctx().remove_path();
                    }
                });
            })
            .unwrap();
        let rect = resp.response.rect;
        {
            let pos = rect.left_top();
            let rect = Rect::from_two_pos(pos, pos + egui::vec2(-30.0, 30.0));
            let ui = &mut ui.child_ui(rect, Layout::bottom_up(Align::Min));
            name.cstr_cs(VISIBLE_DARK, CstrStyle::Bold).label(ui);
        }
        if let Some(hover_content) = self.hover_content {
            if let Some((i, rect)) = hovered_rect {
                if data.entities[i].is_some() {
                    const WIDTH: f32 = 300.0;
                    let (pos, pivot) = if available_rect.right() - rect.right() < WIDTH {
                        (rect.left_center(), Align2::RIGHT_CENTER)
                    } else {
                        (rect.right_center(), Align2::LEFT_CENTER)
                    };
                    Window::new("hover_slot")
                        .title_bar(false)
                        .frame(Frame::none())
                        .max_width(WIDTH)
                        .pivot(pivot)
                        .fixed_pos(pos)
                        .resizable(false)
                        .interactable(false)
                        .constrain_to(ui.ctx().screen_rect())
                        .show(ui.ctx(), |ui| {
                            ui.vertical_centered_justified(|ui| {
                                hover_content(i, data.entities[i], ui, world)
                            })
                        });
                }
            }
        }
        ui.ctx().remove_path();
    }
}

fn show_frame(
    ind: usize,
    max_size: f32,
    overflow: bool,
    name: Option<Cstr>,
    data: &mut UnitContainerData,
    ui: &mut Ui,
) -> Response {
    let size = max_size.min(130.0);
    let (rect, response) = ui.allocate_exact_size(egui::vec2(size, size), Sense::drag());
    data.positions[ind] = rect.center();
    let color = if response.contains_pointer() {
        ui.ctx().set_hovered(rect);
        YELLOW
    } else {
        if overflow {
            RED
        } else {
            VISIBLE_DARK
        }
    };
    if let Some(name) = name {
        ui.allocate_ui_at_rect(
            Rect::from_min_size(rect.left_bottom(), egui::vec2(rect.width(), 20.0)),
            |ui| name.label(ui),
        );
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
    response
}
