use super::*;

#[derive(Clone, Copy)]
pub struct BattleCamera {
    zoom: f32,
    pos: egui::Vec2,
    rect: Rect,
}

impl Default for BattleCamera {
    fn default() -> Self {
        Self {
            zoom: 22.0,
            pos: default(),
            rect: Rect::ZERO,
        }
    }
}

impl BattleCamera {
    fn update(&mut self, ui: &mut Ui) {
        self.rect = ui.available_rect_before_wrap();
    }
    fn u(&self) -> f32 {
        self.rect.width() / self.zoom
    }
    fn center(&self) -> Pos2 {
        self.rect.center() + self.pos / self.zoom
    }
    fn rect_pos(&self, pos: egui::Pos2) -> egui::Pos2 {
        self.center() + pos.to_vec2() * self.u()
    }
    fn zoom_delta(&mut self, delta: f32) {
        if delta == 0.0 {
            return;
        }
        self.zoom += delta * 0.02;
        self.zoom = self.zoom.at_least(3.0);
    }
    fn to_center(&mut self) {
        self.zoom = 22.0;
        self.pos = default();
    }
    pub fn show(bs: &mut BattleSimulation, t: f32, ui: &mut Ui) {
        let mut cam = ui
            .data(|r| r.get_temp::<BattleCamera>(ui.id()))
            .unwrap_or_default();
        cam.update(ui);
        let bg_response = ui.allocate_rect(cam.rect, Sense::click_and_drag());
        if bg_response.dragged() {
            cam.pos += bg_response.drag_delta() * cam.zoom;
        }
        if bg_response.hovered() {
            ui.input(|r| cam.zoom_delta(-r.smooth_scroll_delta.y));
        }
        if bg_response.double_clicked() {
            cam.to_center();
        }

        let slots = global_settings().team_slots;
        for s in 1..=slots {
            cam.show_slot_rect(s as i32, ui);
            cam.show_slot_rect(-(s as i32), ui);
        }

        let fusions: HashSet<Entity> = HashSet::from_iter(
            bs.world
                .query_filtered::<Entity, With<NFusion>>()
                .iter(&bs.world),
        );
        let mut entities: VecDeque<Entity> = bs
            .world
            .query_filtered::<Entity, Without<Parent>>()
            .iter(&bs.world)
            .collect();
        let context = Context::new(&bs.world).set_t(t).take();
        while let Some(entity) = entities.pop_front() {
            let context = context.clone().set_owner(entity).take();
            if context.get_bool(VarName::visible).unwrap_or(true) {
                entities.extend(context.get_children(entity));
                let pos = context
                    .get_var(VarName::position)
                    .unwrap_or_default()
                    .get_vec2()
                    .unwrap()
                    .to_pos2();
                let pos = cam.rect_pos(pos);
                let rect = Rect::from_center_size(pos, cam.u().v2() * 2.0);
                if fusions.contains(&entity) {
                    let fusion = context.get_node::<NFusion>(entity).unwrap();
                    fusion.paint(rect, &context, ui).ui(ui);
                    if ui.rect_contains_pointer(rect) {
                        cursor_window(ui.ctx(), |ui| {
                            fusion.show_card(&context, ui).ui(ui);
                        });
                    }
                } else if let Some(rep) = context.get_node::<NRepresentation>(entity) {
                    rep.pain_or_show_err(rect, &context, ui);
                }
            }
        }

        ui.data_mut(|w| w.insert_temp(ui.id(), cam));
    }
    fn slot_rect(&self, slot: i32) -> Rect {
        Rect::from_center_size(
            self.center() + egui::vec2(slot as f32 * self.u() * 2.0, 0.0),
            self.u().v2() * 2.0,
        )
    }
    fn show_slot_rect(&self, slot: i32, ui: &mut Ui) -> Response {
        let rect = self.slot_rect(slot);
        RectButton::new_rect(rect).ui(ui, |color, rect, _, ui| {
            corners_rounded_rect(
                rect.shrink(rect.width() * 0.1),
                rect.width() * 0.15,
                color.stroke(),
                ui,
            );
        })
    }
}
