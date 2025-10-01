use super::*;

pub struct BattlePlugin;

impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Editor), Self::load_from_client_state)
            .init_resource::<ReloadData>()
            .add_systems(
                FixedUpdate,
                Self::reload.run_if(in_state(GameState::Editor)),
            );
    }
}

#[derive(Resource)]
pub struct BattleData {
    pub teams_world: World,
    pub team_left: Entity,
    pub team_right: Entity,
    pub battle: Battle,
    pub simulation: BattleSimulation,
    pub t: f32,
    pub playback_speed: f32,
    pub playing: bool,
    pub on_done: Option<fn(u64, bool, u64)>,
    pub slot_actions: Vec<(String, fn(i32, Entity, &mut ClientContext))>,
}

#[derive(Resource, Default)]
pub struct ReloadData {
    pub reload_requested: bool,
    pub last_reload: f64,
}

impl BattleData {
    fn load(battle: Battle) -> Self {
        let mut teams_world = World::new();
        let team_left = teams_world.spawn_empty().id();
        let team_right = teams_world.spawn_empty().id();
        Context::from_world_r(&mut teams_world, |context| {
            battle.left.clone().unpack_entity(context, team_left)?;
            battle.right.clone().unpack_entity(context, team_right)
        })
        .unwrap();
        let simulation = BattleSimulation::new(battle.clone()).start();
        Self {
            teams_world,
            team_left,
            team_right,
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
                let mut fusion = NFusion::default();
                fusion.index = team.fusions.get().unwrap().len() as i32;
                fusion.id = next_id();
                fusion.owner = team.owner;
                let mut slot = NFusionSlot::default();
                slot.id = next_id();
                slot.owner = team.owner;
                fusion.slots.get_mut().unwrap().push(slot);
                team.fusions.get_mut().unwrap().push(fusion);
            }
        }
        world.insert_resource(BattleData::load(Battle { left, right, id }));
    }
    pub fn load_from_client_state(world: &mut World) {
        if let Some((left, right)) = pd().client_state.get_battle_test_teams() {
            Self::load_teams(0, left, right, world);
        } else {
            Self::load_teams(0, default(), default(), world);
        };
    }
    pub fn on_done_callback(f: fn(u64, bool, u64), world: &mut World) {
        if let Some(mut r) = world.get_resource_mut::<BattleData>() {
            r.on_done = Some(f);
        }
    }
    fn reload(mut data: ResMut<BattleData>, mut reload: ResMut<ReloadData>) {
        if reload.reload_requested && reload.last_reload + 0.1 < gt().elapsed() {
            reload.reload_requested = false;
            reload.last_reload = gt().elapsed();
            // *data = BattleData::load(data.battle.clone());
            let (left, right) = (data.team_left, data.team_right);
            let (left, right) = Context::from_world_r(&mut data.teams_world, |context| {
                let left = NTeam::pack_entity(context, left)?;
                let right = NTeam::pack_entity(context, right)?;
                Ok((left, right))
            })
            .unwrap();
            data.battle.left = left;
            data.battle.right = right;
            data.playing = false;
            data.t = 0.0;
            data.simulation = BattleSimulation::new(data.battle.clone()).start();
            pd_mut(|pd| {
                pd.client_state
                    .set_battle_test_teams(&data.battle.left, &data.battle.right);
            });
        }
    }
    pub fn open_world_inspector_window(world: &mut World) {
        let mut selected: Option<Entity> = None;
        Window::new("battle world inspector", move |ui, world| {
            let Some(mut bd) = world.get_resource_mut::<BattleData>() else {
                "BattleData not found".cstr().label(ui);
                return;
            };
            let world = &mut bd.simulation.world;
            Context::from_world_r(world, |context| {
                if let Some(entity) = selected {
                    ui.horizontal(|ui| {
                        format!("selected {entity}").label(ui);
                        if "clear".cstr().button(ui).clicked() {
                            selected = None;
                        }
                    });
                    for (var, state) in &context.component::<NodeState>(entity)?.vars {
                        ui.horizontal(|ui| {
                            var.cstr().label(ui);
                            state.value.cstr().label(ui);
                        });
                    }
                    ui.columns_const(|[ui1, ui2]| -> NodeResult<()> {
                        "parents".cstr().label(ui1);
                        "children".cstr().label(ui2);
                        for parent in context.parents_entity(entity)? {
                            let kind = context.component::<NodeState>(parent)?.kind;
                            if format!("{kind} {parent}").button(ui1).clicked() {
                                selected = Some(parent);
                            }
                        }
                        for child in context.children_entity(entity)? {
                            let kind = context.component::<NodeState>(child)?.kind;
                            if format!("{kind} {child}").button(ui2).clicked() {
                                selected = Some(child);
                            }
                        }
                        Ok(())
                    })
                    .ui(ui);
                } else {
                    for (entity, ns) in context
                        .world_mut()?
                        .query::<(Entity, &NodeState)>()
                        .iter(context.world()?)
                    {
                        if format!("{} {}", ns.kind, entity).button(ui).clicked() {
                            selected = Some(entity);
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
            &mut data.teams_world,
            data.team_left,
            data.team_right,
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
        let controls_height = overlay_height - slider_height;

        let overlay_rect = Rect::from_min_size(
            main_rect.left_bottom() - egui::vec2(0.0, overlay_height),
            egui::vec2(main_rect.width(), overlay_height),
        );

        let slider_rect = Rect::from_min_size(
            overlay_rect.left_bottom() - egui::vec2(0.0, slider_height),
            egui::vec2(overlay_rect.width(), slider_height),
        );

        let controls_rect = Rect::from_min_size(
            overlay_rect.min,
            egui::vec2(overlay_rect.width(), controls_height),
        );

        let pointer_pos = ui.ctx().pointer_hover_pos();
        let show_full_controls = pointer_pos.map_or(false, |pos| overlay_rect.contains(pos));

        // Always show progress slider at the bottom
        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(slider_rect), |ui| {
            ui.set_clip_rect(slider_rect);
            if data.simulation.duration > 0.0 {
                let bg_color = ui.visuals().extreme_bg_color.gamma_multiply(0.8);
                ui.painter().rect_filled(slider_rect, 0.0, bg_color);

                Slider::new("ts").name(false).full_width().ui(
                    &mut data.t,
                    0.0..=data.simulation.duration,
                    ui,
                );
            }
        });

        if data.playing {
            data.t += gt().last_delta() * data.playback_speed;
            data.t = data.t.at_most(data.simulation.duration);
        }
        if data.t >= data.simulation.duration && !data.simulation.ended() {
            data.simulation.run();
        }

        // Show full controls if pointer is in overlay area
        if show_full_controls {
            ui.allocate_new_ui(egui::UiBuilder::new().max_rect(controls_rect), |ui| {
                ui.set_clip_rect(controls_rect);
                ui.vertical(|ui| {
                    Self::render_main_controls(ui, data, controls_rect);
                    Self::render_speed_controls(ui, data, controls_rect);
                });
            });
        }
        Ok(())
    }

    fn render_main_controls(ui: &mut Ui, data: &mut BattleData, controls_rect: Rect) {
        let main_btn_size = 35.0;
        let btn_padding = 25.0;
        ui.horizontal(|ui| {
            let center = controls_rect.center_top() + egui::vec2(0.0, main_btn_size * 0.5);
            let rect = Rect::from_center_size(center, main_btn_size.v2());

            // Previous button
            if RectButton::new_rect(rect.translate(egui::vec2(-(main_btn_size + btn_padding), 0.0)))
                .ui(ui, |color, rect, _, ui| {
                    let rect =
                        Rect::from_center_size(rect.center(), rect.size() * egui::vec2(1.0, 0.5));
                    let left = Rect::from_min_max(rect.left_top(), rect.center_bottom());
                    let right = Rect::from_min_max(rect.center_top(), rect.right_bottom());
                    triangle(right, color, 3, ui);
                    ui.painter().line_segment(
                        [left.center_top(), left.center_bottom()],
                        color.stroke_w(6.0),
                    );
                })
                .clicked()
            {
                data.t -= 1.0;
            }

            // Play/Pause button
            if RectButton::new_rect(rect)
                .ui(ui, |color, rect, _, ui| {
                    if !data.playing {
                        triangle(rect, color, 1, ui);
                    } else {
                        let left = Rect::from_min_max(rect.left_top(), rect.center_bottom());
                        let right = Rect::from_min_max(rect.center_top(), rect.right_bottom());
                        ui.painter().line_segment(
                            [left.center_top(), left.center_bottom()],
                            color.stroke_w(10.0),
                        );
                        ui.painter().line_segment(
                            [right.center_top(), right.center_bottom()],
                            color.stroke_w(10.0),
                        );
                    }
                })
                .clicked()
            {
                data.playing = !data.playing;
            }

            // Next button
            if RectButton::new_rect(rect.translate(egui::vec2(main_btn_size + btn_padding, 0.0)))
                .ui(ui, |color, rect, _, ui| {
                    let rect =
                        Rect::from_center_size(rect.center(), rect.size() * egui::vec2(1.0, 0.5));
                    let left = Rect::from_min_max(rect.left_top(), rect.center_bottom());
                    let right = Rect::from_min_max(rect.center_top(), rect.right_bottom());
                    triangle(left, color, 1, ui);
                    ui.painter().line_segment(
                        [right.center_top(), right.center_bottom()],
                        color.stroke_w(6.0),
                    );
                })
                .clicked()
            {
                data.t = data.simulation.duration;
            }
        });
    }

    fn render_speed_controls(ui: &mut Ui, data: &mut BattleData, controls_rect: Rect) {
        let main_btn_size = 35.0;
        let mut rect = ui.available_rect_before_wrap();
        rect.set_top(controls_rect.top() + main_btn_size);

        ui.horizontal(|ui| {
            let btn_size = 25.0;
            let btn_padding = 25.0;

            let center = rect.center_top() + egui::vec2(0.0, btn_size * 0.5);
            let rect = Rect::from_center_size(center, btn_size.v2());

            // Slower button
            if RectButton::new_rect(rect.translate(egui::vec2(-(btn_size + btn_padding), 0.0)))
                .ui(ui, |color, rect, _, ui| {
                    let rect =
                        Rect::from_center_size(rect.center(), rect.size() * egui::vec2(1.0, 0.5));
                    let left = Rect::from_min_max(rect.left_top(), rect.center_bottom());
                    let right = Rect::from_min_max(rect.center_top(), rect.right_bottom());
                    triangle(left, color, 3, ui);
                    triangle(right, color, 3, ui);
                })
                .clicked()
            {
                data.playback_speed = data.playback_speed * 0.5;
            }

            ui.add_space(10.0);

            // Speed display/reset button
            if RectButton::new_rect(rect.translate(egui::vec2(0.0, 5.0)))
                .ui(ui, |color, _, _, ui| {
                    ui.vertical_centered_justified(|ui| {
                        data.playback_speed.cstr_c(color).label(ui)
                    });
                })
                .clicked()
            {
                data.playback_speed = 1.0;
            }

            ui.add_space(10.0);

            // Faster button
            if RectButton::new_rect(rect.translate(egui::vec2(btn_size + btn_padding, 0.0)))
                .ui(ui, |color, rect, _, ui| {
                    let rect =
                        Rect::from_center_size(rect.center(), rect.size() * egui::vec2(1.0, 0.5));
                    let left = Rect::from_min_max(rect.left_top(), rect.center_bottom());
                    let right = Rect::from_min_max(rect.center_top(), rect.right_bottom());
                    triangle(left, color, 1, ui);
                    triangle(right, color, 1, ui);
                })
                .clicked()
            {
                data.playback_speed = data.playback_speed * 2.0;
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

    pub fn pane_edit_graph(_left: bool, _ui: &mut Ui, _world: &mut World) {}
}
