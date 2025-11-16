use super::*;

pub struct BattlePlugin;

impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Battle), Self::on_enter)
            .add_systems(OnExit(GameState::Battle), |world: &mut World| {
                world.remove_resource::<BattleData>();
            });
    }
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
    pub is_replay: bool,
}

impl BattleData {
    fn load(battle: Battle) -> Self {
        let mut source = battle.clone().to_source();

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
            is_replay: false,
        }
    }

    fn load_replay(battle: Battle) -> Self {
        let mut source = battle.clone().to_source();

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
            is_replay: true,
        }
    }
}

impl BattlePlugin {
    fn on_enter(world: &mut World) {
        if world.contains_resource::<BattleData>() {
            return;
        }

        let result = with_solid_source(|ctx| {
            let player_result = player(ctx);

            match player_result {
                Ok(p) => {
                    let mut m_clone = p.active_match_ref(ctx)?.clone();
                    if m_clone.state.is_battle() {
                        if let Some(last) = m_clone
                            .battles_load(ctx)?
                            .into_iter()
                            .sorted_by_key(|b| b.id)
                            .last()
                        {
                            if last.result.is_none() {
                                let left = if let Ok(mut team) = ctx.load::<NTeam>(last.team_left) {
                                    team.load_all(ctx)?.take()
                                } else {
                                    NTeam::default().with_id(next_id())
                                };
                                let right = if let Ok(mut team) = ctx.load::<NTeam>(last.team_right)
                                {
                                    team.load_all(ctx)?.take()
                                } else {
                                    NTeam::default().with_id(next_id())
                                };
                                Ok(Some((last.id, Some((left, right)), false)))
                            } else {
                                Ok(None)
                            }
                        } else {
                            Ok(None)
                        }
                    } else {
                        Ok(None)
                    }
                }
                Err(_) => Ok(None),
            }
        });

        match result {
            Ok(Some((id, teams, _is_active_battle))) => {
                if let Some((left, right)) = teams {
                    Self::load_teams(id, left, right, world);
                    Self::on_done_callback(
                        |id, result, hash| {
                            cn().reducers
                                .match_submit_battle_result(id, result, hash)
                                .notify_op();
                        },
                        world,
                    );
                } else {
                    error!("Failed to get teams for battle");
                    GameState::Shop.set_next(world);
                }
            }
            Ok(None) => {
                debug!("No active battle, go back to Shop");
                GameState::Shop.set_next(world);
            }
            Err(e) => {
                e.cstr().notify_error(world);
                GameState::Shop.set_next(world);
            }
        }
    }

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

    pub fn load_replay(mut left: NTeam, mut right: NTeam, world: &mut World) {
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
        let id = next_id();
        world.insert_resource(BattleData::load_replay(Battle { left, right, id }));
    }

    pub fn on_done_callback(f: fn(u64, bool, u64), world: &mut World) {
        if let Some(mut r) = world.get_resource_mut::<BattleData>() {
            r.on_done = Some(f);
        }
    }

    pub fn pane_view(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        let mut data = world
            .remove_resource::<BattleData>()
            .to_custom_e("No battle loaded")?;

        let t = data.t;
        let main_rect = ui.available_rect_before_wrap();

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
        if !Self::render_end_screen(ui, &mut data, main_rect, world)? {
            Self::render_battle_texts(ui, &data, main_rect).ui(ui);
            Self::render_player_names(ui, &data, main_rect).ui(ui);
        }

        world.insert_resource(data);
        Ok(())
    }

    fn render_battle_texts(ui: &mut Ui, data: &BattleData, rect: Rect) -> NodeResult<()> {
        let sim = data.source.battle()?;
        let t = data.source.t().unwrap();
        let ui = &mut ui.new_child(UiBuilder::new().max_rect(Rect::from_center_size(
            rect.center_top(),
            egui::vec2(200.0, 0.0),
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
        Ok(())
    }

    fn render_player_names(ui: &mut Ui, data: &BattleData, rect: Rect) -> NodeResult<()> {
        let sim = data.source.battle()?;
        let left = sim.battle.left.owner;
        let right = sim.battle.right.owner;
        const SIZE: egui::Vec2 = egui::vec2(150.0, LINE_HEIGHT);
        with_solid_source(|ctx| {
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
            Ok(())
        })
    }

    fn render_playback_controls(
        ui: &mut Ui,
        data: &mut BattleData,
        main_rect: Rect,
    ) -> NodeResult<()> {
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
            if ui.button("⏮").clicked() {
                data.t = 0.0;
            }

            if ui.button("⏪").clicked() {
                data.t = (data.t - 1.0).max(0.0);
            }

            if ui.button(if data.playing { "⏸" } else { "▶" }).clicked() {
                data.playing = !data.playing;
            }

            if ui.button("⏩").clicked() {
                let duration = data
                    .source
                    .exec_context(|ctx| ctx.battle().map(|s| s.duration).unwrap_or(0.0));
                data.t = (data.t + 1.0).min(duration);
            }

            if ui.button("⏭").clicked() {
                let duration = data
                    .source
                    .exec_context(|ctx| ctx.battle().map(|s| s.duration).unwrap_or(0.0));
                data.t = duration;
            }

            ui.separator();

            ui.label("Speed:");

            ui.add(
                DragValue::new(&mut data.playback_speed)
                    .speed(0.1)
                    .range(0.1..=8.0),
            );

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

    fn render_end_screen(
        ui: &mut Ui,
        data: &mut BattleData,
        main_rect: Rect,
        world: &mut World,
    ) -> NodeResult<bool> {
        let (duration, ended, result) = data.source.exec_context_ref(|ctx| {
            let sim = ctx.battle().unwrap();
            (sim.duration, sim.ended(), sim.fusions_right.is_empty())
        });

        if data.t >= duration && ended {
            ui.scope_builder(UiBuilder::new().max_rect(main_rect), |ui| {
                ui.vertical_centered_justified(|ui| {
                    if result {
                        "Victory".cstr_cs(GREEN, CstrStyle::Heading)
                    } else {
                        "Defeat".cstr_cs(RED, CstrStyle::Heading)
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
                        if "Complete".cstr().button(ui).clicked() {
                            if data.is_replay {
                                GameState::Title.set_next(world);
                            } else if let Some(on_done) = data.on_done.take() {
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
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
