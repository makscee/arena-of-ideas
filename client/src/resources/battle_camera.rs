use bevy_egui::egui::UiKind;

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
        let pos = context
            .get_var(VarName::position)
            .get_vec2()
            .track()?
            .to_pos2();
        let pos = self.rect_pos(pos);
        Ok(Rect::from_center_size(pos, self.u().v2() * 2.0))
    }
    pub fn show(ctx: &mut ClientContext, t: f32, ui: &mut Ui) {
        let sim = ctx.battle().unwrap();
        Self::show_with_actions(ctx, t, ui, &Vec::new(), sim.team_left, sim.team_right);
    }

    pub fn show_with_actions(
        ctx: &mut ClientContext,
        t: f32,
        ui: &mut Ui,
        slot_actions: &Vec<(String, fn(i32, Entity, &mut ClientContext))>,
        team_left_id: u64,
        team_right_id: u64,
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
            cam.show_slot_with_action(s as i32, ui, slot_actions, ctx, team_left_id, team_right_id);
            cam.show_slot_with_action(
                -(s as i32),
                ui,
                slot_actions,
                ctx,
                team_left_id,
                team_right_id,
            );
        }
        ctx.exec_mut(|ctx| {
            *ctx.source_mut().t_mut().unwrap() = t;
            let world = ctx.world_mut()?;
            for entity in world
                .query_filtered::<Entity, With<NUnitRepresentation>>()
                .iter(world)
                .collect_vec()
            {
                let ids = entity.ids(&ctx)?;
                let id = ids
                    .into_iter()
                    .next()
                    .ok_or(NodeError::entity_not_found(entity.index() as u64))?;
                ctx.with_owner(id, |ctx| {
                    if !ctx.get_var(VarName::visible).get_bool().unwrap_or_default() {
                        return Ok(());
                    }
                    let rect = cam.rect_from_context(ctx).track()?;
                    ctx.load::<NUnitRepresentation>(id)
                        .track()?
                        .material
                        .paint(rect, ctx, ui);
                    Ok(())
                })
                .track()
                .notify_error_op();
            }
            let world = ctx.world_mut()?;
            for entity in world
                .query_filtered::<Entity, (With<NStatusRepresentation>, Without<NHouse>)>()
                .iter(world)
                .collect_vec()
            {
                let ids = entity.ids(&ctx)?;
                let id = ids
                    .into_iter()
                    .next()
                    .ok_or(NodeError::entity_not_found(entity.index() as u64))?;
                ctx.with_status(id, |ctx| {
                    if !ctx.get_var(VarName::visible).get_bool().unwrap_or_default() {
                        return Ok(());
                    }
                    let rect = cam.rect_from_context(ctx).track()?;
                    ctx.load::<NStatusRepresentation>(id)
                        .track()?
                        .material
                        .paint(rect, ctx, ui);
                    Ok(())
                })
                .notify_error_op();
            }
            let world = ctx.world_mut()?;
            for fusion in world.query::<&NFusion>().iter(world).cloned().collect_vec() {
                ctx.with_owner(fusion.id, |context| {
                    if !context
                        .get_var(VarName::visible)
                        .get_bool()
                        .unwrap_or_default()
                    {
                        return Ok(());
                    }
                    let rect = cam.rect_from_context(context).track()?;
                    fusion.paint(rect, context, ui)?;
                    Ok(())
                })
                .notify_error_op();
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
        ctx: &mut ClientContext,
        team_left_id: u64,
        team_right_id: u64,
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
            let team_id = if slot < 0 {
                team_left_id
            } else {
                team_right_id
            };
            response.bar_menu(|ui| {
                for (action_name, action_fn) in slot_actions {
                    if ui.button(action_name).clicked() {
                        if let Ok(team_entity) = ctx.entity(team_id) {
                            action_fn(slot, team_entity, ctx);
                        }

                        ui.close_kind(UiKind::Menu);
                    }
                }
            });
        }
    }
}
