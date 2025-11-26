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

pub struct BattleCameraBuilder<'a> {
    filled_slot_actions: Vec<(
        String,
        Box<dyn Fn(u64, u64, i32, &mut ClientContext, &mut Ui) + 'a>,
    )>,
    empty_slot_actions: Vec<(
        String,
        Box<dyn Fn(u64, i32, &mut ClientContext, &mut Ui) + 'a>,
    )>,
    on_hover_filled: Option<Box<dyn Fn(u64, &mut ClientContext, &mut Ui) + 'a>>,
    on_drag_drop: Option<Box<dyn Fn(u64, i32, u64, i32, &mut ClientContext) -> bool + 'a>>,
    on_battle_end: Option<Box<dyn Fn(&mut Ui, bool) + 'a>>,
}

impl<'a> BattleCameraBuilder<'a> {
    pub fn new() -> Self {
        Self {
            filled_slot_actions: Vec::new(),
            empty_slot_actions: Vec::new(),
            on_hover_filled: None,
            on_drag_drop: None,
            on_battle_end: None,
        }
    }

    pub fn filled_slot_action<F>(mut self, name: String, action: F) -> Self
    where
        F: Fn(u64, u64, i32, &mut ClientContext, &mut Ui) + 'a,
    {
        self.filled_slot_actions.push((name, Box::new(action)));
        self
    }

    pub fn empty_slot_action<F>(mut self, name: String, action: F) -> Self
    where
        F: Fn(u64, i32, &mut ClientContext, &mut Ui) + 'a,
    {
        self.empty_slot_actions.push((name, Box::new(action)));
        self
    }

    pub fn on_hover_filled<F>(mut self, callback: F) -> Self
    where
        F: Fn(u64, &mut ClientContext, &mut Ui) + 'a,
    {
        self.on_hover_filled = Some(Box::new(callback));
        self
    }

    pub fn on_drag_drop<F>(mut self, callback: F) -> Self
    where
        F: Fn(u64, i32, u64, i32, &mut ClientContext) -> bool + 'a,
    {
        self.on_drag_drop = Some(Box::new(callback));
        self
    }

    pub fn on_battle_end<F>(mut self, callback: F) -> Self
    where
        F: Fn(&mut Ui, bool) + 'a,
    {
        self.on_battle_end = Some(Box::new(callback));
        self
    }

    pub fn show(mut self, battle_data: &mut BattleData, ui: &mut Ui) {
        let has_on_done = battle_data.on_done.is_some();
        let should_show_end_screen = std::cell::Cell::new(false);

        battle_data.source.exec_context(|ctx| {
            let sim = ctx.battle().unwrap();
            let team_left_id = sim.team_left;
            let team_right_id = sim.team_right;
            let duration = sim.duration;
            let ended = sim.ended();

            if battle_data.t >= duration && ended {
                should_show_end_screen.set(true);
                return;
            }

            let mut cam = ui
                .data(|r| r.get_temp::<BattleCamera>(ui.id()))
                .unwrap_or_default();
            cam.update(ui, &mut battle_data.playing);

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

            self.render_battle_texts(ctx, ui, cam.rect, battle_data.t);
            self.render_player_names(ctx, ui, cam.rect);
            self.render_playback_controls(
                ui,
                cam.rect,
                &mut battle_data.t,
                duration,
                &mut battle_data.playing,
                &mut battle_data.playback_speed,
            );

            let slots = global_settings().team_slots;
            for s in 1..=slots {
                cam.show_slot(ctx, s as i32, ui, &mut self, team_right_id);
                cam.show_slot(ctx, -(s as i32), ui, &mut self, team_left_id);
            }

            ctx.exec_mut(|ctx| {
                *ctx.source_mut().t_mut().unwrap() = battle_data.t;
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
                for slot in world
                    .query::<&NTeamSlot>()
                    .iter(world)
                    .cloned()
                    .collect_vec()
                {
                    let Ok(unit) = slot.unit_ref(ctx).cloned() else {
                        continue;
                    };
                    ctx.with_owner(unit.id, |ctx| {
                        if !ctx.get_var(VarName::visible).get_bool().unwrap_or_default() {
                            return Ok(());
                        }
                        let rect = cam.rect_from_context(ctx).track()?;
                        unit.representation_ref(ctx)?.material.paint(rect, ctx, ui);
                        unit.show_status_tags(rect, ctx, ui).ui(ui);
                        Ok(())
                    })
                    .ui(ui);
                }

                let world = ctx.world_mut()?;
                for entity in world
                    .query_filtered::<Entity, With<NRepresentation>>()
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
                        ctx.load::<NRepresentation>(id)
                            .track()?
                            .material
                            .paint(rect, ctx, ui);
                        Ok(())
                    })
                    .track()
                    .notify_error_op();
                }
                Ok(())
            })
            .ui(ui);

            ui.data_mut(|w| w.insert_temp(ui.id(), cam));
        });

        if should_show_end_screen.get() {
            let result = battle_data
                .source
                .exec_context_ref(|ctx| ctx.battle().unwrap().units_right.is_empty());
            self.render_end_screen(ui, result, has_on_done);
            self.render_end_screen_with_actions(ui, battle_data);
        }
    }

    fn render_end_screen(&self, ui: &mut Ui, result: bool, has_on_done: bool) {
        ui.vertical_centered_justified(|ui| {
            if result {
                "Victory".cstr_cs(GREEN, CstrStyle::Heading)
            } else {
                "Defeat".cstr_cs(RED, CstrStyle::Heading)
            }
            .label(ui);
        });

        if let Some(on_end) = &self.on_battle_end {
            on_end(ui, result);
        }
    }

    fn render_end_screen_with_actions(&self, ui: &mut Ui, battle_data: &mut BattleData) {
        ui.columns(2, |ui| {
            ui[0].vertical_centered_justified(|ui| {
                ui.set_max_width(200.0);
                if "Replay".cstr().button(ui).clicked() {
                    battle_data.t = 0.0;
                }
            });

            if battle_data.on_done.is_some() {
                ui[1].vertical_centered_justified(|ui| {
                    ui.set_max_width(200.0);
                    if "Complete".cstr().button(ui).clicked() {
                        if let Some(on_done) = battle_data.on_done.take() {
                            let hash = battle_data.source.exec_context_ref(|ctx| {
                                let sim = ctx.battle().unwrap();
                                let mut h = std::collections::hash_map::DefaultHasher::new();
                                use std::hash::Hash;
                                for a in &sim.log.actions {
                                    a.hash(&mut h);
                                }
                                use std::hash::Hasher;
                                h.finish()
                            });
                            let result = battle_data.source.exec_context_ref(|ctx| {
                                ctx.battle().unwrap().units_right.is_empty()
                            });
                            on_done(battle_data.battle.id, result, hash);
                        }
                    }
                });
            }
        });
    }

    fn render_battle_texts(&mut self, ctx: &mut ClientContext, ui: &mut Ui, rect: Rect, t: f32) {
        let sim = ctx.battle().unwrap();
        let ui = &mut ui.new_child(UiBuilder::new().max_rect(Rect::from_center_size(
            rect.center_top(),
            egui::vec2(200.0, 100.0),
        )));
        ui.vertical_centered(|ui| {
            if let Some(text) = sim.get_text_at(BattleText::CurrentEvent, t) {
                text.cstr().label(ui);
            }
            if let Some(text) = sim.get_text_at(BattleText::Turn, t) {
                text.cstr().label(ui);
            }
            if let Some(text) = sim.get_text_at(BattleText::Fatigue, t) {
                text.cstr().label(ui);
            }
        });
    }

    fn render_player_names(&mut self, ctx: &mut ClientContext, ui: &mut Ui, rect: Rect) {
        let sim = ctx.battle().unwrap();
        let left = sim.battle.left.owner;
        let right = sim.battle.right.owner;
        const SIZE: egui::Vec2 = egui::vec2(150.0, LINE_HEIGHT);

        if let Ok(player) = ctx.load::<NPlayer>(left) {
            let ui = &mut ui.new_child(
                UiBuilder::new()
                    .max_rect(Rect::from_min_max(rect.left_top(), rect.left_top() + SIZE)),
            );
            ui.vertical_centered(|ui| {
                player.player_name.cstr().label(ui);
                format!("[s [tw {}]]", player.id())
                    .as_label(ui.style())
                    .selectable(true)
                    .ui(ui);
            });
        }
        if let Ok(player) = ctx.load::<NPlayer>(right) {
            let ui = &mut ui.new_child(UiBuilder::new().max_rect(Rect::from_min_max(
                rect.right_top() + SIZE * egui::vec2(-1.0, 1.0),
                rect.right_top(),
            )));
            ui.vertical_centered(|ui| {
                player.player_name.cstr().label(ui);
                format!("[s [tw {}]]", player.id())
                    .as_label(ui.style())
                    .selectable(true)
                    .ui(ui);
            });
        }
    }

    fn render_playback_controls(
        &mut self,
        ui: &mut Ui,
        rect: Rect,
        t: &mut f32,
        duration: f32,
        playing: &mut bool,
        speed: &mut f32,
    ) {
        let overlay_height = 80.0;
        let slider_rect = Rect::from_min_size(
            rect.left_bottom() - egui::vec2(0.0, overlay_height),
            egui::vec2(rect.width(), overlay_height),
        );

        ui.scope_builder(egui::UiBuilder::new().max_rect(slider_rect), |ui| {
            ui.horizontal(|ui| {
                if ui.button("⏮").clicked() {
                    *t = 0.0;
                }

                if ui.button("⏪").clicked() {
                    *t = (*t - 1.0).max(0.0);
                }

                if ui.button(if *playing { "⏸" } else { "▶" }).clicked() {
                    *playing = !*playing;
                }

                if ui.button("⏩").clicked() {
                    *t = (*t + 1.0).min(duration);
                }

                if ui.button("⏭").clicked() {
                    *t = duration;
                }

                ui.separator();

                ui.label("Speed:");

                ui.add(DragValue::new(speed).speed(0.1).range(0.1..=8.0));

                let speeds = [1.0, 2.0, 4.0];
                for &s in &speeds {
                    let is_active = (*speed - s).abs() < 0.01;
                    if ui.selectable_label(is_active, format!("x{}", s)).clicked() {
                        *speed = s;
                    }
                }

                ui.separator();

                if duration > 0.0 {
                    Slider::new("time")
                        .name(false)
                        .full_width()
                        .ui(t, 0.0..=duration, ui);
                }
            });
        });
    }
}

impl BattleCamera {
    pub fn builder() -> BattleCameraBuilder<'static> {
        BattleCameraBuilder::new()
    }

    fn update(&mut self, ui: &mut Ui, _playing: &mut bool) {
        self.rect = ui.available_rect_before_wrap();
    }

    fn u(&self) -> f32 {
        self.rect.width() / self.zoom
    }

    fn center(&self) -> egui::Pos2 {
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

    fn slot_rect(&self, slot: i32) -> Rect {
        Rect::from_center_size(
            self.center() + egui::vec2(slot as f32 * self.u() * 2.0, 0.0),
            self.u().v2() * 2.0,
        )
    }

    fn show_slot(
        &mut self,
        ctx: &mut ClientContext,
        slot: i32,
        ui: &mut Ui,
        builder: &mut BattleCameraBuilder,
        team_id: u64,
    ) {
        let rect = self.slot_rect(slot);
        let response = ui.allocate_rect(rect, Sense::click());

        if let Ok(team) = ctx.load::<NTeam>(team_id) {
            if let Ok(slots) = team.slots.get() {
                if let Some(team_slot) = slots.iter().find(|s| s.index == slot) {
                    if let Ok(unit) = team_slot.unit_ref(ctx).cloned() {
                        if response.hovered() {
                            if let Some(on_hover) = &builder.on_hover_filled {
                                on_hover(unit.id, ctx, ui);
                            }
                        }

                        for (_name, action) in &builder.filled_slot_actions {
                            if response.clicked() {
                                action(team_id, unit.id, slot, ctx, ui);
                            }
                        }

                        if let Some(on_drag_drop) = &builder.on_drag_drop {
                            if response.drag_started() {
                                ui.memory_mut(|mem| {
                                    mem.data.insert_temp(response.id, (team_id, unit.id, slot));
                                });
                            }

                            if let Some(drop_target) =
                                ui.memory(|mem| mem.data.get_temp::<(u64, u64, i32)>(ui.id()))
                            {
                                if response.hovered() && ui.input(|i| i.pointer.any_released()) {
                                    let (from_team, from_unit, from_slot) = drop_target;
                                    on_drag_drop(from_unit, from_slot, unit.id, slot, ctx);
                                    ui.memory_mut(|mem| {
                                        mem.data.remove::<(u64, u64, i32)>(ui.id());
                                    });
                                }
                            }
                        }
                    } else {
                        for (_name, action) in &builder.empty_slot_actions {
                            if response.clicked() {
                                action(team_id, slot, ctx, ui);
                            }
                        }
                    }
                }
            }
        }
    }
}
