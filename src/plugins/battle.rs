use bevy::app::FixedUpdate;
use bevy_egui::egui::epaint::PathStroke;

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
struct BattleData {
    teams_world: World,
    team_left: Entity,
    team_right: Entity,
    battle: Battle,
    simulation: BattleSimulation,
    t: f32,
    playback_speed: f32,
    playing: bool,
    on_done: Option<Box<dyn Fn(u64, bool, u64) + Sync + Send>>,
}

#[derive(Resource, Default)]
struct ReloadData {
    reload_requested: bool,
    last_reload: f64,
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
            playing: false,
            playback_speed: 1.0,
            on_done: None,
        }
    }
}

impl BattlePlugin {
    pub fn load_teams(id: u64, left: NTeam, right: NTeam, world: &mut World) {
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
            *data = BattleData::load(data.battle.clone());
            pd_mut(|pd| {
                pd.client_state
                    .set_battle_test_teams(&data.battle.left, &data.battle.right);
            });
        }
    }
    pub fn pane_view(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        let mut data = world
            .remove_resource::<BattleData>()
            .to_custom_e("No battle loaded")?;

        let t = data.t;
        BattleCamera::show(&mut data.simulation, t, ui);
        world.insert_resource(data);
        Ok(())
    }
    pub fn pane_controls(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        let mut data = world
            .remove_resource::<BattleData>()
            .to_custom_e("No battle loaded")?;
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
        if simulation.duration > 0.0 {
            Slider::new("ts")
                .full_width()
                .ui(t, 0.0..=simulation.duration, ui);
        }
        let mut rect = ui.available_rect_before_wrap();
        let btn_size = 30.0;
        rect.set_height(btn_size);
        fn triangle(rect: Rect, color: Color32, pointed_right: bool, ui: &mut Ui) {
            ui.painter()
                .add(egui::Shape::Path(PathShape::convex_polygon(
                    if pointed_right {
                        [rect.left_top(), rect.right_center(), rect.left_bottom()]
                    } else {
                        [rect.right_top(), rect.right_bottom(), rect.left_center()]
                    }
                    .into(),
                    color,
                    PathStroke::NONE,
                )));
        }
        if RectButton::new_rect(Rect::from_center_size(rect.center(), btn_size.v2()))
            .ui(ui, |color, rect, _, ui| {
                if !*playing {
                    triangle(rect, color, true, ui);
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
            *playing = !*playing;
        }
        if RectButton::new_rect(Rect::from_center_size(
            rect.center() + egui::vec2(btn_size * 2.0, 0.0),
            btn_size.v2(),
        ))
        .ui(ui, |color, rect, _, ui| {
            let rect = Rect::from_center_size(rect.center(), rect.size() * egui::vec2(1.0, 0.5));
            let left = Rect::from_min_max(rect.left_top(), rect.center_bottom());
            let right = Rect::from_min_max(rect.center_top(), rect.right_bottom());
            triangle(left, color, true, ui);
            ui.painter().line_segment(
                [right.center_top(), right.center_bottom()],
                color.stroke_w(6.0),
            );
        })
        .clicked()
        {
            *t = simulation.duration;
        }
        if RectButton::new_rect(Rect::from_center_size(
            rect.center() - egui::vec2(btn_size * 2.0, 0.0),
            btn_size.v2(),
        ))
        .ui(ui, |color, rect, _, ui| {
            let rect = Rect::from_center_size(rect.center(), rect.size() * egui::vec2(1.0, 0.5));
            let left = Rect::from_min_max(rect.left_top(), rect.center_bottom());
            let right = Rect::from_min_max(rect.center_top(), rect.right_bottom());
            triangle(right, color, false, ui);
            ui.painter().line_segment(
                [left.center_top(), left.center_bottom()],
                color.stroke_w(6.0),
            );
        })
        .clicked()
        {
            *t -= 1.0;
        }
        ui.advance_cursor_after_rect(rect);

        if *t >= simulation.duration && simulation.ended() {
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
        } else {
            let rect = rect.translate(egui::vec2(0.0, rect.height()));
            if RectButton::new_rect(Rect::from_center_size(rect.center(), btn_size.v2()))
                .ui(ui, |color, _, _, ui| {
                    ui.horizontal_centered(|ui| playback_speed.cstr_c(color).label(ui));
                })
                .clicked()
            {
                *playback_speed = 1.0;
            }
            if RectButton::new_rect(Rect::from_center_size(
                rect.center() + egui::vec2(btn_size * 2.0, 0.0),
                btn_size.v2(),
            ))
            .ui(ui, |color, rect, _, ui| {
                let rect =
                    Rect::from_center_size(rect.center(), rect.size() * egui::vec2(1.0, 0.5));
                let left = Rect::from_min_max(rect.left_top(), rect.center_bottom());
                let right = Rect::from_min_max(rect.center_top(), rect.right_bottom());
                triangle(left, color, true, ui);
                triangle(right, color, true, ui);
            })
            .clicked()
            {
                *playback_speed = *playback_speed * 2.0;
            }
            if RectButton::new_rect(Rect::from_center_size(
                rect.center() - egui::vec2(btn_size * 2.0, 0.0),
                btn_size.v2(),
            ))
            .ui(ui, |color, rect, _, ui| {
                let rect =
                    Rect::from_center_size(rect.center(), rect.size() * egui::vec2(1.0, 0.5));
                let left = Rect::from_min_max(rect.left_top(), rect.center_bottom());
                let right = Rect::from_min_max(rect.center_top(), rect.right_bottom());
                triangle(left, color, false, ui);
                triangle(right, color, false, ui);
            })
            .clicked()
            {
                *playback_speed = *playback_speed * 0.5;
            }
            ui.advance_cursor_after_rect(rect);

            ui.horizontal(|ui| {
                if "+1".cstr().button(ui).clicked() {
                    simulation.run();
                }
                if "+10".cstr().button(ui).clicked() {
                    for _ in 0..10 {
                        simulation.run();
                    }
                }
                if "+100".cstr().button(ui).clicked() {
                    for _ in 0..100 {
                        simulation.run();
                    }
                }
            });
            if *playing {
                *t += gt().last_delta() * *playback_speed;
                *t = t.at_most(simulation.duration);
            }
            if *t >= simulation.duration && !simulation.ended() {
                simulation.run();
            }
        }
        world.insert_resource(data);
        Ok(())
    }
    pub fn pane_edit_graph(left: bool, ui: &mut Ui, world: &mut World) {
        world.resource_scope(|world, mut data: Mut<BattleData>| {
            Context::from_world_r(&mut data.teams_world, |context| {
                let exp_kind_id = Id::new("explorer kind");
                let mut kind = ui.ctx().data_mut(|w| {
                    w.get_temp_mut_or::<NodeKind>(exp_kind_id, NodeKind::NHouse)
                        .clone()
                });
                if Selector::new("kind").ui_enum(&mut kind, ui) {
                    ui.ctx().data_mut(|w| w.insert_temp(exp_kind_id, kind));
                }
                kind.show_explorer(context, ui)
            })
            .ui(ui);
            Context::from_world(world, |context| {
                let team = if left {
                    &mut data.battle.left
                } else {
                    &mut data.battle.right
                };

                if team
                    .view_with_children_mut(ViewContextNew::new(ui), context, ui)
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
            let mut add_fusion = false;
            let mut add_unit: Option<(u64, u64)> = None;
            let mut remove_unit: Option<(u64, u64)> = None;
            Context::from_world_r(teams_world, |context| {
                NFusion::slots_editor(
                    team_entity,
                    context,
                    ui,
                    |ui| {
                        ui.vertical_centered_justified(|ui| {
                            if "add fusion".cstr().button(ui).clicked() {
                                add_fusion = true;
                            }
                        });
                    },
                    |fusion| {
                        edited = Some(fusion);
                    },
                    |fusion, unit| {
                        add_unit = Some((fusion.id, unit));
                    },
                    |fusion, unit| {
                        remove_unit = Some((fusion.id, unit));
                    },
                )
                .ui(ui);
                if add_fusion {
                    let entity = context.world_mut().unwrap().spawn_empty().id();
                    let team = NTeam::get(team_entity, context).unwrap();
                    let slot = team.fusions_load(context).len() as i32;
                    let mut fusion = NFusion::default();
                    context.link_parent_child(team.id, fusion.id).log();
                    fusion.slot = slot;
                    fusion.unpack_entity(context, entity).log();
                    changed = true;
                }
                if let Some(fusion) = edited {
                    context
                        .world_mut()
                        .unwrap()
                        .entity_mut(fusion.entity())
                        .insert(fusion);
                    changed = true;
                }
                if let Some((fusion, unit)) = add_unit {
                    context.link_parent_child(unit, fusion)?;
                    changed = true;
                }
                if let Some((fusion, unit)) = remove_unit {
                    context.get_by_id_mut::<NFusion>(fusion)?.remove_unit(unit);
                    context.unlink_parent_child(unit, fusion)?;
                    changed = true;
                }

                if changed {
                    world.resource_mut::<ReloadData>().reload_requested = true;
                    let updated_team = NTeam::pack_entity(context, team_entity).unwrap();
                    dbg!(&updated_team);
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
