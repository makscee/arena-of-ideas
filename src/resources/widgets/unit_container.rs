use egui::{NumExt, Sense, Window};

use super::*;

pub struct UnitContainer {
    faction: Faction,
    slots: usize,
    max_slots: usize,
    right_to_left: bool,
    hug_unit: bool,
    show_name: bool,
    on_swap: Option<Box<dyn Fn(usize, usize, &mut World) + Send + Sync>>,
    top_content: Option<Box<dyn FnOnce(&mut Ui, &mut World) + Send + Sync>>,
    slot_content: Option<Box<dyn Fn(usize, Option<Entity>, &mut Ui, &mut World) + Send + Sync>>,
    hover_content: Option<Box<dyn Fn(usize, Option<Entity>, &mut Ui, &mut World) + Send + Sync>>,
    slot_name: HashMap<usize, String>,
    pivot: Align2,
    position: egui::Vec2,
    min_size: f32,
}

#[derive(Resource, Default)]
pub struct UnitContainerResource {
    containers: HashMap<Faction, UnitContainerData>,
}

#[derive(Debug)]
struct UnitContainerData {
    positions: Vec<Pos2>,
    entities: Vec<Option<Entity>>,
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
            show_name: false,
            top_content: None,
            slot_content: None,
            hover_content: None,
            on_swap: None,
            pivot: Align2::CENTER_CENTER,
            position: default(),
            slot_name: default(),
            min_size: 10.0,
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
    pub fn min_size(mut self, value: f32) -> Self {
        self.min_size = value;
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
    pub fn name(mut self) -> Self {
        self.show_name = true;
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
    pub fn on_swap(
        mut self,
        action: impl Fn(usize, usize, &mut World) + Send + Sync + 'static,
    ) -> Self {
        self.on_swap = Some(Box::new(action));
        self
    }
    pub fn slot_name(mut self, i: usize, name: String) -> Self {
        self.slot_name.insert(i, name);
        self
    }
    pub fn ui(self, ui: &mut Ui, world: &mut World) {
        let mut data = world
            .resource_mut::<UnitContainerResource>()
            .containers
            .remove(&self.faction)
            .unwrap_or_default();
        data.positions.resize(self.slots, default());
        data.entities.resize(self.slots, None);
        if ui.available_width() > ui.style().spacing.item_spacing.x * self.max_slots as f32 {
            ui.columns(self.max_slots, |ui| {
                for (ind, ui) in ui.iter_mut().enumerate() {
                    Self::show_unit_frame(ind, ui);
                    if let Some(content) = &self.slot_content {
                        (content)(ind, None, ui, world);
                    }
                }
            });
        }
    }
    fn show_unit_frame(ind: usize, ui: &mut Ui) -> Response {
        let rect = ui.available_rect_before_wrap();
        let size = rect.size().x.min(rect.size().y);
        let rect = Rect::from_center_size(rect.center(), egui::Vec2::splat(size));
        let resp = ui.allocate_rect(rect, Sense::hover());
        let color = if resp.hovered() { YELLOW } else { VISIBLE_DARK };
        let stroke = Stroke { width: 1.0, color };
        let ind_rect = Rect::from_min_max(
            rect.right_top() + egui::vec2(-10.0, 5.0),
            rect.right_top() + egui::vec2(-5.0, 0.0),
        );
        {
            let ui = &mut ui.child_ui(ind_rect, Layout::top_down(Align::Max), None);
            ind.to_string().cstr_cs(color, CstrStyle::Bold).label(ui);
        }
        const DASH_COUNT: f32 = 5.0;
        let dash_size = size / (DASH_COUNT + (DASH_COUNT - 1.0) * 0.5);
        let gap_size = dash_size * 0.5;
        ui.painter().add(egui::Shape::dashed_line(
            &[rect.left_top(), rect.right_top()],
            stroke,
            dash_size,
            gap_size,
        ));
        ui.painter().add(egui::Shape::dashed_line(
            &[rect.right_top(), rect.right_bottom()],
            stroke,
            dash_size,
            gap_size,
        ));
        ui.painter().add(egui::Shape::dashed_line(
            &[rect.right_bottom(), rect.left_bottom()],
            stroke,
            dash_size,
            gap_size,
        ));
        ui.painter().add(egui::Shape::dashed_line(
            &[rect.left_bottom(), rect.left_top()],
            stroke,
            dash_size,
            gap_size,
        ));
        resp
    }
    pub fn ui_old(self, ui: &mut Ui, world: &mut World) {
        let mut data = world
            .resource_mut::<UnitContainerResource>()
            .containers
            .remove(&self.faction)
            .unwrap_or_default();
        data.positions.resize(self.slots + 1, default());
        data.entities.resize(self.slots + 1, None);
        let name = format!("{}", self.faction);
        ui.ctx().add_path(&name);
        const MARGIN: Margin = Margin::same(8.0);
        let available_rect = ui.available_rect_before_wrap();
        let pos = available_rect.min + self.position * available_rect.size();
        let max_size = if self.hug_unit {
            CameraPlugin::pixel_unit(ui.ctx(), world) * 2.5
        } else {
            (available_rect.width() / self.slots as f32 - MARGIN.left - MARGIN.right).min(130.0)
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
            .default_width(1.0)
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
                            let response = show_frame(
                                i,
                                max_size.at_least(self.min_size),
                                i >= self.max_slots,
                                name,
                                &mut data,
                                ui,
                            );
                            if let Some(action) = &self.on_swap {
                                if response.dragged() {
                                    if let Some(pointer) = ui.ctx().pointer_latest_pos() {
                                        let origin = response.rect.center();
                                        ui.set_clip_rect(ui.ctx().screen_rect());
                                        ui.painter().arrow(
                                            origin,
                                            pointer.to_vec2() - origin.to_vec2(),
                                            Stroke {
                                                width: 3.0,
                                                color: YELLOW,
                                            },
                                        )
                                    }
                                }
                                response.dnd_set_drag_payload(i);
                                if let Some(drop_i) = response.dnd_release_payload::<usize>() {
                                    if i != *drop_i {
                                        action(*drop_i, i, world);
                                    }
                                }
                            }
                            if let Some(name) = self.slot_name.get(&i) {
                                let ui = &mut ui.child_ui(
                                    Rect::from_two_pos(
                                        response.rect.left_top(),
                                        response.rect.right_top() + egui::vec2(0.0, -20.0),
                                    ),
                                    Layout::left_to_right(Align::Center),
                                    None,
                                );
                                name.cstr().label(ui);
                            }
                            if response.hovered() && ui.ctx().dragged_id().is_none() {
                                hovered_rect = Some((i, response.rect));
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
        if self.show_name {
            let pos = rect.left_top();
            let rect = Rect::from_two_pos(pos, pos + egui::vec2(-30.0, 30.0));
            let ui = &mut ui.child_ui(rect, Layout::bottom_up(Align::Min), None);
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
        world
            .resource_mut::<UnitContainerResource>()
            .containers
            .insert(self.faction, data);
    }

    pub fn place_into_slots(world: &mut World) {
        let Some(cam_entity) = CameraPlugin::get_entity(world) else {
            return;
        };
        let delta = delta_time(world);
        let units = UnitPlugin::collect_factions([Faction::Shop, Faction::Team].into(), world);
        let mut data = world.remove_resource::<UnitContainerResource>().unwrap();
        let camera = world.get::<Camera>(cam_entity).unwrap().clone();
        let transform = world.get::<GlobalTransform>(cam_entity).unwrap().clone();
        for cd in data.containers.values_mut() {
            for e in cd.entities.iter_mut() {
                *e = None;
            }
        }
        for (entity, faction) in units {
            if let Some(cd) = data.containers.get_mut(&faction) {
                let context = Context::new(entity);
                let slot = context.get_int(VarName::Slot, world).unwrap();
                let position = context.get_vec2(VarName::Position, world).unwrap();
                let slot = slot as usize;
                let need_pos = cd
                    .positions
                    .get(slot)
                    .map(|p| screen_to_world_cam(p.to_bvec2(), &camera, &transform))
                    .unwrap_or_default();
                cd.entities[slot] = Some(entity);
                let mut state = VarState::get_mut(entity, world);
                state.change_vec2(VarName::Position, (need_pos - position) * delta * 5.0);
            }
        }
        world.insert_resource(data);
    }
}

fn show_frame(
    ind: usize,
    size: f32,
    overflow: bool,
    name: Option<Cstr>,
    data: &mut UnitContainerData,
    ui: &mut Ui,
) -> Response {
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
        let ui = &mut ui.child_ui(ind_rect, Layout::top_down(Align::Max), None);
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
