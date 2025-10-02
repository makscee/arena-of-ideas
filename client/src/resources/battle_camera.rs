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
    fn rect_from_context(&self, context: &ClientContext) -> Result<Rect, NodeError> {
        let pos = context.get_vec2(VarName::position)?.to_pos2();
        let pos = self.rect_pos(pos);
        Ok(Rect::from_center_size(pos, self.u().v2() * 2.0))
    }
    pub fn show(bs: &mut BattleSimulation, t: f32, ui: &mut Ui) {
        Self::show_with_actions(
            bs,
            t,
            ui,
            &Vec::new(),
            &mut World::default(),
            Entity::PLACEHOLDER,
            Entity::PLACEHOLDER,
        );
    }

    pub fn show_with_actions(
        bs: &mut BattleSimulation,
        t: f32,
        ui: &mut Ui,
        slot_actions: &Vec<(String, fn(i32, Entity, &mut ClientContext))>,
        teams_world: &mut World,
        team_left: Entity,
        team_right: Entity,
    ) {
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
            cam.show_slot_with_action(
                s as i32,
                ui,
                slot_actions,
                teams_world,
                team_left,
                team_right,
            );
            cam.show_slot_with_action(
                -(s as i32),
                ui,
                slot_actions,
                teams_world,
                team_left,
                team_right,
            );
        }
        bs.world
            .with_context_mut(|context| {
                todo!();
                // context.t = Some(t);
                let world = context.world_mut()?;
                for fusion in world.query::<&NFusion>().iter(world).cloned().collect_vec() {
                    context
                        .with_owner(fusion.id, |context| {
                            if !context.get_bool(VarName::visible)? {
                                return Ok(());
                            }
                            let rect = cam.rect_from_context(context)?;
                            fusion.paint(rect, &context, ui)?;
                            if ui.rect_contains_pointer(rect) {
                                cursor_window(ui.ctx(), |ui| {
                                    fusion.as_card().compose(context, ui);
                                    Ok(())
                                });
                            }
                            Ok(())
                        })
                        .ui(ui);
                }
                let world = context.world_mut()?;
                for rep in world
                    .query::<&NUnitRepresentation>()
                    .iter(world)
                    .collect_vec()
                {
                    context
                        .with_owner(rep.id, |context| {
                            if !context.get_bool(VarName::visible)? {
                                return Ok(());
                            }
                            let rect = cam.rect_from_context(context)?;
                            rep.material.paint(rect, context, ui)
                        })
                        .ui(ui);
                }
                let world = context.world_mut()?;
                for rep in world
                    .query::<&NStatusRepresentation>()
                    .iter(world)
                    .collect_vec()
                {
                    context
                        .with_owner(rep.id, |context| {
                            if !context.get_bool(VarName::visible)? {
                                return Ok(());
                            }
                            let rect = cam.rect_from_context(context)?;
                            rep.material.paint(rect, context, ui)
                        })
                        .ui(ui);
                }
                Ok(())
            })
            .ui(ui);

        ui.data_mut(|w| w.insert_temp(ui.id(), cam));
    }
    fn slot_rect(&self, slot: i32) -> Rect {
        Rect::from_center_size(
            self.center() + egui::vec2(slot as f32 * self.u() * 2.0, 0.0),
            self.u().v2() * 2.0,
        )
    }

    fn show_slot_with_action(
        &self,
        slot: i32,
        ui: &mut Ui,
        slot_actions: &Vec<(String, fn(i32, Entity, &mut ClientContext))>,
        teams_world: &mut World,
        team_left: Entity,
        team_right: Entity,
    ) {
        let rect = self.slot_rect(slot);
        let response = RectButton::new_rect(rect).ui(ui, |color, rect, _, ui| {
            corners_rounded_rect(
                rect.shrink(rect.width() * 0.1),
                rect.width() * 0.15,
                color.stroke(),
                ui,
            );
        });

        if !slot_actions.is_empty() {
            let team_entity = if slot < 0 { team_left } else { team_right };
            response.bar_menu(|ui| {
                for (action_name, action_fn) in slot_actions {
                    if ui.button(action_name).clicked() {
                        teams_world.with_context_mut(|context| {
                            action_fn(slot, team_entity, context);
                            Ok(())
                        });
                        ui.close_menu();
                    }
                }
            });
        }
    }
}
