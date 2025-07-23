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
    pub on_done: Option<Box<dyn Fn(u64, bool, u64) + Sync + Send>>,
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
        }
    }
}

impl BattlePlugin {
    pub fn load_teams(id: u64, mut left: NTeam, mut right: NTeam, world: &mut World) {
        let slots = global_settings().team_slots as usize;
        for team in [&mut left, &mut right] {
            while team.fusions.len() < slots {
                let mut fusion = NFusion::default();
                fusion.slot = team.fusions.len() as i32;
                fusion.id = next_id();
                fusion.owner = team.owner;
                fusion.lvl = 1;
                team.fusions.push(fusion);
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
    pub fn on_done_callback(f: impl Fn(u64, bool, u64) + Send + Sync + 'static, world: &mut World) {
        if let Some(mut r) = world.get_resource_mut::<BattleData>() {
            r.on_done = Some(Box::new(f));
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
                    for (var, state) in &context.get::<NodeState>(entity)?.vars {
                        ui.horizontal(|ui| {
                            var.cstr().label(ui);
                            state.value.cstr().label(ui);
                        });
                    }
                    ui.columns_const(|[ui1, ui2]| -> Result<(), ExpressionError> {
                        "parents".cstr().label(ui1);
                        "children".cstr().label(ui2);
                        for parent in context.parents_entity(entity)? {
                            let kind = context.get::<NodeState>(parent)?.kind;
                            if format!("{kind} {parent}").button(ui1).clicked() {
                                selected = Some(parent);
                            }
                        }
                        for child in context.children_entity(entity)? {
                            let kind = context.get::<NodeState>(child)?.kind;
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
    pub fn pane_view(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        let mut data = world
            .remove_resource::<BattleData>()
            .to_custom_e("No battle loaded")?;

        let t = data.t;
        let main_rect = ui.available_rect_before_wrap();

        // Show battle camera in the full area
        BattleCamera::show(&mut data.simulation, t, ui);

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
        let BattleData {
            teams_world: _,
            battle,
            simulation,
            t,
            playing,
            team_left: _,
            team_right: _,
            playback_speed,
            on_done,
        } = &mut data;

        if *playing {
            *t += gt().last_delta() * *playback_speed;
            *t = t.at_most(simulation.duration);
        }
        if *t >= simulation.duration && !simulation.ended() {
            simulation.run();
        }
        // Show full controls if pointer is in overlay area
        if show_full_controls {
            ui.allocate_new_ui(egui::UiBuilder::new().max_rect(controls_rect), |ui| {
                ui.set_clip_rect(controls_rect);
                ui.vertical(|ui| {
                    // Main control buttons
                    let main_btn_size = 35.0;
                    let btn_padding = 25.0;
                    ui.horizontal(|ui| {
                        let center =
                            controls_rect.center_top() + egui::vec2(0.0, main_btn_size * 0.5);
                        let rect = Rect::from_center_size(center, main_btn_size.v2());

                        // Previous button
                        if RectButton::new_rect(
                            rect.translate(egui::vec2(-(main_btn_size + btn_padding), 0.0)),
                        )
                        .ui(ui, |color, rect, _, ui| {
                            let rect = Rect::from_center_size(
                                rect.center(),
                                rect.size() * egui::vec2(1.0, 0.5),
                            );
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
                            *t -= 1.0;
                        }

                        // Play/Pause button
                        if RectButton::new_rect(rect)
                            .ui(ui, |color, rect, _, ui| {
                                if !*playing {
                                    triangle(rect, color, 1, ui);
                                } else {
                                    let left =
                                        Rect::from_min_max(rect.left_top(), rect.center_bottom());
                                    let right =
                                        Rect::from_min_max(rect.center_top(), rect.right_bottom());
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
                            *playing = !*playing;
                        }

                        // Next button
                        if RectButton::new_rect(
                            rect.translate(egui::vec2(main_btn_size + btn_padding, 0.0)),
                        )
                        .ui(ui, |color, rect, _, ui| {
                            let rect = Rect::from_center_size(
                                rect.center(),
                                rect.size() * egui::vec2(1.0, 0.5),
                            );
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
                            *t = simulation.duration;
                        }
                    });

                    let mut rect = ui.available_rect_before_wrap();
                    rect.set_top(controls_rect.top() + main_btn_size);
                    // Playback speed controls
                    ui.horizontal(|ui| {
                        let btn_size = 25.0;

                        let center = rect.center_top() + egui::vec2(0.0, btn_size * 0.5);
                        let rect = Rect::from_center_size(center, btn_size.v2());
                        // Slower button
                        if RectButton::new_rect(
                            rect.translate(egui::vec2(-(btn_size + btn_padding), 0.0)),
                        )
                        .ui(ui, |color, rect, _, ui| {
                            let rect = Rect::from_center_size(
                                rect.center(),
                                rect.size() * egui::vec2(1.0, 0.5),
                            );
                            let left = Rect::from_min_max(rect.left_top(), rect.center_bottom());
                            let right = Rect::from_min_max(rect.center_top(), rect.right_bottom());
                            triangle(left, color, 3, ui);
                            triangle(right, color, 3, ui);
                        })
                        .clicked()
                        {
                            *playback_speed = *playback_speed * 0.5;
                        }

                        ui.add_space(10.0);

                        // Speed display/reset button
                        if RectButton::new_rect(rect.translate(egui::vec2(0.0, 5.0)))
                            .ui(ui, |color, _, _, ui| {
                                ui.vertical_centered_justified(|ui| {
                                    playback_speed.cstr_c(color).label(ui)
                                });
                            })
                            .clicked()
                        {
                            *playback_speed = 1.0;
                        }

                        ui.add_space(10.0);

                        // Faster button
                        if RectButton::new_rect(
                            rect.translate(egui::vec2(btn_size + btn_padding, 0.0)),
                        )
                        .ui(ui, |color, rect, _, ui| {
                            let rect = Rect::from_center_size(
                                rect.center(),
                                rect.size() * egui::vec2(1.0, 0.5),
                            );
                            let left = Rect::from_min_max(rect.left_top(), rect.center_bottom());
                            let right = Rect::from_min_max(rect.center_top(), rect.right_bottom());
                            triangle(left, color, 1, ui);
                            triangle(right, color, 1, ui);
                        })
                        .clicked()
                        {
                            *playback_speed = *playback_speed * 2.0;
                        }
                    });
                });
            });
        }
        if *t >= simulation.duration && simulation.ended() {
            ui.scope_builder(UiBuilder::new().max_rect(main_rect), |ui| {
                let result = simulation.fusions_right.is_empty();
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
                            *t = 0.0;
                        }
                    });
                    ui[1].vertical_centered_justified(|ui| {
                        ui.set_max_width(200.0);
                        if let Some(on_done) = on_done {
                            if "Complete".cstr().button(ui).clicked() {
                                let mut h = DefaultHasher::new();
                                for a in &simulation.log.actions {
                                    a.hash(&mut h);
                                }
                                on_done(battle.id, result, h.finish());
                            }
                        }
                    });
                })
            });
        }

        world.insert_resource(data);
        Ok(())
    }

    pub fn pane_edit_graph(left: bool, ui: &mut Ui, world: &mut World) {
        world.resource_scope(|world, mut data: Mut<BattleData>| {
            Context::from_world(world, |context| {
                let team = if left {
                    &mut data.battle.left
                } else {
                    &mut data.battle.right
                };

                if team
                    .view_with_children_mut(ViewContext::new(ui), context, ui)
                    .changed
                    || team.view_mut(ViewContext::new(ui), context, ui).changed
                {
                    context
                        .world_mut()
                        .unwrap()
                        .resource_mut::<ReloadData>()
                        .reload_requested = true;
                }
            });
        });
    }
    pub fn pane_edit_slots(left: bool, ui: &mut Ui, world: &mut World) {
        world.resource_scope(|world, mut data: Mut<BattleData>| {
            let BattleData {
                teams_world,
                team_left,
                team_right,
                battle,
                simulation: _,
                t: _,
                playing: _,
                playback_speed: _,
                on_done: _,
            } = data.as_mut();
            let mut changed = false;
            let team_entity = if left { *team_left } else { *team_right };
            let mut edited = None;
            let mut add_unit: Option<(u64, u64)> = None;
            let mut remove_unit: Option<(u64, u64)> = None;
            Context::from_world_r(teams_world, |context| {
                NFusion::slots_editor(
                    team_entity,
                    context,
                    ui,
                    |_, _, _| {},
                    |fusion| {
                        edited = Some(fusion);
                    },
                    |fusion, unit| {
                        add_unit = Some((fusion.id, unit));
                    },
                    |fusion, unit| {
                        remove_unit = Some((fusion.id, unit));
                    },
                    |_| {},
                )
                .ui(ui);
                if let Some(fusion) = edited {
                    context
                        .world_mut()
                        .unwrap()
                        .entity_mut(fusion.entity())
                        .insert(fusion);
                    changed = true;
                }
                if let Some((fusion, unit)) = add_unit {
                    context.get_by_id_mut::<NFusion>(fusion)?.action_limit += 10;
                    context.link_parent_child(unit, fusion)?;
                    changed = true;
                }
                if let Some((fusion, unit)) = remove_unit {
                    // First get the unit index
                    let unit_index = {
                        let fusion_ref = context.get_by_id::<NFusion>(fusion)?;
                        let units = fusion_ref.units(context)?;
                        units.iter().position(|u| u.id == unit)
                    };

                    // Then remove from fusion behavior
                    if let Some(unit_index) = unit_index {
                        let mut fusion_ref = context.get_by_id_mut::<NFusion>(fusion)?;
                        if unit_index < fusion_ref.behavior.len() {
                            fusion_ref.behavior.remove(unit_index);
                        }
                    }

                    // Finally unlink the unit
                    context.unlink_parent_child(unit, fusion)?;
                    changed = true;
                }

                if changed {
                    world.resource_mut::<ReloadData>().reload_requested = true;
                    let updated_team = NTeam::pack_entity(context, team_entity).unwrap();
                    let team = if left {
                        &mut battle.left
                    } else {
                        &mut battle.right
                    };
                    *team = updated_team;
                }
                Ok(())
            })
            .ui(ui);
        });
    }
}
