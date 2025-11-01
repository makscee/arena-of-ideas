use super::*;

pub struct BattlePlugin;

impl Plugin for BattlePlugin {
    fn build(&self, _app: &mut App) {}
}

#[derive(Resource)]
pub struct BattleData {
    pub battle: Battle,
    pub source: Sources<'static>,
    pub t: f32,
    pub playback_speed: f32,
    pub playing: bool,
    pub on_done: Option<fn(u64, bool, u64)>,
    pub slot_actions: Vec<(String, fn(i32, Entity, &mut ClientContext))>,
}

impl BattleData {
    fn load(battle: Battle) -> Self {
        let mut source = battle.clone().to_source();

        // Start the simulation
        source
            .exec_context(|ctx| BattleSimulation::start(ctx))
            .unwrap();

        Self {
            battle,
            source,
            t: 0.0,
            playing: true,
            playback_speed: 1.0,
            on_done: None,
            slot_actions: Vec::new(),
        }
    }
}

impl BattlePlugin {
    pub fn load_teams(id: u64, mut left: NTeam, mut right: NTeam, world: &mut World) {
        let slots = global_settings().team_slots as usize;
        for team in [&mut left, &mut right] {
            if !team.fusions.is_loaded() {
                team.fusions_set(default()).unwrap();
            }
            while team.fusions.get().unwrap().len() < slots {
                let mut fusion = NFusion::default().with_id(next_id());
                fusion.index = team.fusions.get().unwrap().len() as i32;
                fusion.owner = team.owner;
                let mut slot = NFusionSlot::default().with_id(next_id());
                slot.owner = team.owner;
                fusion.slots.set_loaded([slot].into()).ok();
                team.fusions.get_mut().unwrap().push(fusion);
            }
        }
        world.insert_resource(BattleData::load(Battle { left, right, id }));
    }
    pub fn on_done_callback(f: fn(u64, bool, u64), world: &mut World) {
        if let Some(mut r) = world.get_resource_mut::<BattleData>() {
            r.on_done = Some(f);
        }
    }
    pub fn open_world_inspector_window(world: &mut World) {
        let mut selected: Option<u64> = None;
        Window::new("battle world inspector", move |ui, world| {
            let Some(bd) = world.get_resource_mut::<BattleData>() else {
                "BattleData not found".cstr().label(ui);
                return;
            };
            bd.source
                .exec_context_ref(|ctx| {
                    if let Some(id) = selected {
                        ui.horizontal(|ui| {
                            format!("selected {id}").label(ui);
                            if "clear".cstr().button(ui).clicked() {
                                selected = None;
                            }
                        });
                        for (var, state) in &ctx
                            .world()?
                            .get::<NodeStateHistory>(ctx.entity(id)?)
                            .to_not_found()?
                            .vars
                        {
                            ui.horizontal(|ui| {
                                var.cstr().label(ui);
                                state.value.cstr().label(ui);
                            });
                        }
                        ui.columns_const(|[ui1, ui2]| -> NodeResult<()> {
                            "parents".cstr().label(ui1);
                            "children".cstr().label(ui2);
                            for parent in ctx.get_children(id)? {
                                let kind = ctx.get_kind(parent)?;
                                if format!("{kind} {parent}").button(ui1).clicked() {
                                    selected = Some(parent);
                                }
                            }
                            for child in ctx.get_children(id)? {
                                let kind = ctx.get_kind(child)?;
                                if format!("{kind} {child}").button(ui2).clicked() {
                                    selected = Some(child);
                                }
                            }
                            Ok(())
                        })
                        .ui(ui);
                    } else {
                        for (entity, _ns) in ctx
                            .world_mut()?
                            .query::<(Entity, &NodeStateHistory)>()
                            .iter(ctx.world()?)
                        {
                            if format!("{}", entity).button(ui).clicked() {
                                let ids = entity.ids(&ctx)?;
                                if let Some(id) = ids.into_iter().next() {
                                    selected = Some(id);
                                }
                            }
                        }
                    }
                    Ok(())
                })
                .ui(ui);
        })
        .push(world);
    }
    pub fn pane_view(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        let mut data = world
            .remove_resource::<BattleData>()
            .to_custom_e("No battle loaded")?;

        let t = data.t;
        let main_rect = ui.available_rect_before_wrap();

        // Show battle camera with slot actions
        data.source
            .exec_context(|ctx| {
                BattleCamera::show_with_actions(
                    ctx,
                    t,
                    ui,
                    &data.slot_actions,
                    data.battle.left.id(),
                    data.battle.right.id(),
                );
                Ok(())
            })
            .log();

        Self::render_playback_controls(ui, &mut data, main_rect)?;
        Self::render_end_screen(ui, &mut data, main_rect)?;

        world.insert_resource(data);
        Ok(())
    }

    fn render_playback_controls(
        ui: &mut Ui,
        data: &mut BattleData,
        main_rect: Rect,
    ) -> NodeResult<()> {
        // Create overlay for controls at the bottom
        let overlay_height = 80.0;
        let slider_height = 20.0;

        let overlay_rect = Rect::from_min_size(
            main_rect.left_bottom() - egui::vec2(0.0, overlay_height),
            egui::vec2(main_rect.width(), overlay_height),
        );

        let slider_rect = Rect::from_min_size(
            overlay_rect.left_bottom() - egui::vec2(0.0, slider_height),
            egui::vec2(overlay_rect.width(), slider_height),
        );

        ui.scope_builder(egui::UiBuilder::new().max_rect(slider_rect), |ui| {
            Self::render_controls(ui, data);
        });

        let duration = data
            .source
            .exec_context_ref(|ctx| ctx.battle().map(|s| s.duration).unwrap_or(0.0));
        let ended = data
            .source
            .exec_context_ref(|ctx| ctx.battle().map(|s| s.ended()).unwrap_or(true));

        if data.playing {
            data.t += gt().last_delta() * data.playback_speed;
            data.t = data.t.at_most(duration);
        }
        if data.t >= duration && !ended {
            data.source
                .exec_context(|ctx| BattleSimulation::run(ctx))
                .log();
        }
        Ok(())
    }

    fn render_controls(ui: &mut Ui, data: &mut BattleData) {
        ui.horizontal(|ui| {
            // Reset button
            if ui.button("⏮").clicked() {
                data.t = 0.0;
            }

            // Step back
            if ui.button("⏪").clicked() {
                data.t = (data.t - 1.0).max(0.0);
            }

            // Play/Pause button
            if ui.button(if data.playing { "⏸" } else { "▶" }).clicked() {
                data.playing = !data.playing;
            }

            // Step forward
            if ui.button("⏩").clicked() {
                let duration = data
                    .source
                    .exec_context(|ctx| ctx.battle().map(|s| s.duration).unwrap_or(0.0));
                data.t = (data.t + 1.0).min(duration);
            }

            // Jump to end
            if ui.button("⏭").clicked() {
                let duration = data
                    .source
                    .exec_context(|ctx| ctx.battle().map(|s| s.duration).unwrap_or(0.0));
                data.t = duration;
            }

            ui.separator();

            // Speed controls
            ui.label("Speed:");

            ui.add(
                DragValue::new(&mut data.playback_speed)
                    .speed(0.1)
                    .range(0.1..=8.0),
            );

            // Speed preset buttons
            let speeds = [1.0, 2.0, 4.0];
            for &speed in &speeds {
                let is_active = (data.playback_speed - speed).abs() < 0.01;
                if ui
                    .selectable_label(is_active, format!("x{}", speed))
                    .clicked()
                {
                    data.playback_speed = speed;
                }
            }

            ui.separator();

            // Duration slider
            let duration = data
                .source
                .exec_context_ref(|ctx| ctx.battle().map(|s| s.duration).unwrap_or(0.0));
            if duration > 0.0 {
                Slider::new("time")
                    .name(false)
                    .full_width()
                    .ui(&mut data.t, 0.0..=duration, ui);
            }
        });
    }

    fn render_end_screen(ui: &mut Ui, data: &mut BattleData, main_rect: Rect) -> NodeResult<()> {
        let (duration, ended, result) = data.source.exec_context_ref(|ctx| {
            let sim = ctx.battle().unwrap();
            (sim.duration, sim.ended(), sim.fusions_right.is_empty())
        });

        if data.t >= duration && ended {
            ui.scope_builder(UiBuilder::new().max_rect(main_rect), |ui| {
                ui.vertical_centered_justified(|ui| {
                    if result {
                        "Victory".cstr_cs(GREEN, CstrStyle::Bold)
                    } else {
                        "Defeat".cstr_cs(RED, CstrStyle::Bold)
                    }
                    .label(ui);
                });
                ui.columns(2, |ui| {
                    ui[0].vertical_centered_justified(|ui| {
                        ui.set_max_width(200.0);
                        if "Replay".cstr().button(ui).clicked() {
                            data.t = 0.0;
                        }
                    });
                    ui[1].vertical_centered_justified(|ui| {
                        ui.set_max_width(200.0);
                        if let Some(on_done) = data.on_done {
                            if "Complete".cstr().button(ui).clicked() {
                                let hash = data.source.exec_context_ref(|ctx| {
                                    let mut h = DefaultHasher::new();
                                    for a in &ctx.battle().unwrap().log.actions {
                                        a.hash(&mut h);
                                    }
                                    h.finish()
                                });
                                on_done(data.battle.id, result, hash);
                            }
                        }
                    });
                })
            });
        }
        Ok(())
    }
}
