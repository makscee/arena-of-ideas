use super::*;

pub struct TeamContainer {
    faction: Faction,
    slots: usize,
    max_slots: usize,
    right_to_left: bool,
    show_name: bool,
    on_swap: Option<Box<dyn Fn(usize, usize, &mut World) + Send + Sync>>,
    top_content: Option<Box<dyn FnOnce(&mut Ui, &mut World) + Send + Sync>>,
    slot_content: Option<Box<dyn Fn(usize, Option<Entity>, &mut Ui, &mut World) + Send + Sync>>,
    hover_content: Option<Box<dyn Fn(usize, Option<Entity>, &mut Ui, &mut World) + Send + Sync>>,
    slot_name: HashMap<usize, String>,
    empty_slot_text: Option<Cstr>,
}

#[derive(Resource, Default)]
pub struct TeamContainerResource {
    containers: HashMap<Faction, TeamContainerData>,
}

#[derive(Debug)]
struct TeamContainerData {
    positions: Vec<Pos2>,
    entities: Vec<Option<Entity>>,
}

impl Default for TeamContainerData {
    fn default() -> Self {
        Self {
            positions: vec![pos2(0.0, 0.0)],
            entities: vec![None],
        }
    }
}

impl TeamContainer {
    pub fn new(faction: Faction) -> Self {
        Self {
            faction,
            slots: 5,
            max_slots: 5,
            right_to_left: false,
            show_name: false,
            top_content: None,
            slot_content: None,
            hover_content: None,
            on_swap: None,
            slot_name: default(),
            empty_slot_text: None,
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
    pub fn name(mut self) -> Self {
        self.show_name = true;
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
    pub fn empty_slot_text(mut self, text: Cstr) -> Self {
        self.empty_slot_text = Some(text);
        self
    }
    pub fn ui(self, ui: &mut Ui, world: &mut World) {
        let mut data = world
            .resource_mut::<TeamContainerResource>()
            .containers
            .remove(&self.faction)
            .unwrap_or_default();
        data.positions.resize(self.slots, default());
        data.entities.resize(self.slots, None);
        if let Some(content) = self.top_content {
            (content)(ui, world);
        }

        let size = CameraPlugin::pixel_unit(ui.ctx(), world) * 1.3;
        let size = size.at_most(ui.available_width() / self.slots as f32 * 0.5);
        ui.columns(self.slots, |ui| {
            for (i, ui) in ui.iter_mut().rev().enumerate() {
                let resp = Self::show_unit_frame(i, self.max_slots, size, ui);
                if let Some(name) = self.slot_name.get(&i) {
                    let ui = &mut ui.child_ui(
                        Rect::from_center_size(
                            resp.rect.center_top(),
                            egui::vec2(resp.rect.width(), 0.0),
                        )
                        .translate(egui::vec2(0.0, -20.0)),
                        Layout::left_to_right(Align::Max),
                        None,
                    );
                    name.cstr_cs(VISIBLE_DARK, CstrStyle::Bold).label(ui);
                }
                data.positions[i] = resp.rect.center();
                if let Some(entity) = data.entities[i] {
                    ui.vertical_centered_justified(|ui| {
                        entity_name(entity).label(ui);
                    });
                    if let Some(action) = &self.on_swap {
                        if resp.dragged() {
                            if let Some(pointer) = ui.ctx().pointer_latest_pos() {
                                let origin = resp.rect.center();
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
                        resp.dnd_set_drag_payload(i);
                        if let Some(drop_i) = resp.dnd_release_payload::<usize>() {
                            if i != *drop_i {
                                debug!("swap {drop_i} {i}");
                                action(*drop_i, i, world);
                            }
                        }
                    }
                    if resp.hovered() && ui.ctx().dragged_id().is_none() {
                        cursor_window(ui.ctx(), |ui| {
                            match UnitCard::new(&Context::new(entity), world) {
                                Ok(c) => c.ui(ui),
                                Err(e) => error!("{e}"),
                            }
                        });
                    }
                } else if let Some(text) = self.empty_slot_text.as_ref() {
                    let ui = &mut ui.child_ui(
                        resp.rect,
                        Layout::centered_and_justified(egui::Direction::TopDown),
                        None,
                    );
                    text.label(ui);
                }
                if let Some(content) = &self.slot_content {
                    ui.vertical_centered_justified(|ui| {
                        (content)(i, data.entities[i], ui, world);
                    });
                }
            }
        });

        world
            .resource_mut::<TeamContainerResource>()
            .containers
            .insert(self.faction, data);
    }
    fn show_unit_frame(ind: usize, max_slots: usize, size: f32, ui: &mut Ui) -> Response {
        let rect = ui.available_rect_before_wrap();
        let rect = Rect::from_center_size(rect.center_top(), egui::Vec2::ZERO)
            .expand2(egui::vec2(size, 0.0))
            .with_max_y(rect.center_top().y + size * 2.0);
        let resp = ui.allocate_rect(rect, Sense::drag());
        let color = if resp.hovered() {
            YELLOW
        } else if ind >= max_slots {
            DARK_RED
        } else {
            VISIBLE_DARK
        };
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

    pub fn place_into_slots(world: &mut World) {
        let Some(cam_entity) = CameraPlugin::get_entity(world) else {
            return;
        };
        let delta = delta_time(world);
        let units = UnitPlugin::collect_factions([Faction::Shop, Faction::Team].into(), world);
        let mut data = world.remove_resource::<TeamContainerResource>().unwrap();
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
                let slot = context.get_int(VarName::Slot, world).unwrap() as usize;
                let position = context.get_vec2(VarName::Position, world).unwrap();
                let need_pos = cd
                    .positions
                    .get(slot)
                    .map(|p| screen_to_world_cam(p.to_bvec2(), &camera, &transform))
                    .unwrap_or_default();
                if cd.entities.len() > slot {
                    cd.entities[slot] = Some(entity);
                }
                let mut state = VarState::get_mut(entity, world);
                state.change_vec2(VarName::Position, (need_pos - position) * delta * 5.0);
            }
        }
        world.insert_resource(data);
    }
}
