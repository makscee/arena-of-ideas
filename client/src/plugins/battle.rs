use super::*;

pub struct BattlePlugin;

impl Plugin for BattlePlugin {
    fn build(&self, _app: &mut App) {}
}

#[derive(Resource)]
pub struct BattleData {
    pub battle: Battle,
    pub simulation: BattleSimulation,
    pub t: f32,
    pub playback_speed: f32,
    pub playing: bool,
    pub on_done: Option<fn(u64, bool, u64)>,
    pub slot_actions: Vec<(String, fn(i32, Entity, &mut ClientContext))>,
}

impl BattleData {
    fn load(battle: Battle) -> Self {
        let simulation = BattleSimulation::new(battle.clone()).start();
        Self {
            battle,
            simulation,
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
            while team.fusions.get().unwrap().len() < slots {
                let mut fusion = NFusion::default().with_id(next_id());
                fusion.index = team.fusions.get().unwrap().len() as i32;
                fusion.owner = team.owner;
                let mut slot = NFusionSlot::default().with_id(next_id());
                slot.owner = team.owner;
                fusion.slots.state_mut().set([slot].into());
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
            let Some(mut bd) = world.get_resource_mut::<BattleData>() else {
                "BattleData not found".cstr().label(ui);
                return;
            };
            let world = &mut bd.simulation.world;
            world
                .with_context_mut(|ctx| {
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
                                selected = Some(ctx.id(entity)?);
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
        BattleCamera::show_with_actions(
            &mut data.simulation,
            t,
            ui,
            &data.slot_actions,
            data.battle.left.id(),
            data.battle.right.id(),
        );

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

        if data.playing {
            data.t += gt().last_delta() * data.playback_speed;
            data.t = data.t.at_most(data.simulation.duration);
        }
        if data.t >= data.simulation.duration && !data.simulation.ended() {
            data.simulation.run();
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
                data.t = (data.t + 1.0).min(data.simulation.duration);
            }

            // Jump to end
            if ui.button("⏭").clicked() {
                data.t = data.simulation.duration;
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
            if data.simulation.duration > 0.0 {
                Slider::new("time").name(false).full_width().ui(
                    &mut data.t,
                    0.0..=data.simulation.duration,
                    ui,
                );
            }
        });
    }

    fn render_end_screen(ui: &mut Ui, data: &mut BattleData, main_rect: Rect) -> NodeResult<()> {
        if data.t >= data.simulation.duration && data.simulation.ended() {
            ui.scope_builder(UiBuilder::new().max_rect(main_rect), |ui| {
                let result = data.simulation.fusions_right.is_empty();
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
                                let mut h = DefaultHasher::new();
                                for a in &data.simulation.log.actions {
                                    a.hash(&mut h);
                                }
                                on_done(data.battle.id, result, h.finish());
                            }
                        }
                    });
                })
            });
        }
        Ok(())
    }

    pub fn pane_edit_graph(left: bool, ui: &mut Ui, world: &mut World) {
        if let Some(mut state) = world.get_resource_mut::<BattleEditorState>() {
            let needs_reload = if left {
                state.left_team.render_recursive(ui)
            } else {
                state.right_team.render_recursive(ui)
            };

            if needs_reload {
                BattleEditorPlugin::save_changes_and_reload(world);
            }
        } else {
            "[red [b BattleEditoState] not found]".cstr().label(ui);
        }
    }
}
