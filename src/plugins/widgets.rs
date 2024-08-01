use egui::Area;

use super::*;

pub struct WidgetsPlugin;

impl Plugin for WidgetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, Self::ui)
            .init_resource::<WidgetsState>();
    }
}

#[derive(Default, Resource)]
struct WidgetsState {
    arena_normal: Option<Vec<TArenaLeaderboard>>,
    arena_daily: Option<Vec<TArenaLeaderboard>>,
    settings: Option<()>,
    profile: Option<ProfileEditData>,
}

impl WidgetsPlugin {
    pub fn reset_state(world: &mut World) {
        *world.resource_mut::<WidgetsState>() = default();
    }
    fn ui(world: &mut World) {
        let Some(ctx) = &egui_context(world) else {
            return;
        };
        Area::new(Id::new("top_right_info"))
            .anchor(Align2::RIGHT_TOP, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.with_layout(Layout::right_to_left(egui::Align::Min), |ui| {
                    ui.add_space(13.0);
                    if let Some(fps) = world
                        .resource::<DiagnosticsStore>()
                        .get(&FrameTimeDiagnosticsPlugin::FPS)
                    {
                        if let Some(fps) = fps.smoothed() {
                            ui.label(format!("fps: {fps:.0}"));
                        }
                    }
                    ui.label(format!("arena-of-ideas {VERSION}"));
                })
            });

        SectionMenu::default().show(ctx, world);

        let state = cur_state(world);
        let mut ws = world.remove_resource::<WidgetsState>().unwrap();

        // Tiles
        Tile::show_all_tiles(ctx, world);
        match state {
            GameState::Title => {
                Tile::left("Arena Normal").title().close_btn().show_data(
                    &mut ws.arena_normal,
                    ctx,
                    |_, ui| {
                        br(ui);
                        if Button::click("Start new".into()).ui(ui).clicked() {
                            run_start_normal();
                            once_on_run_start_normal(|_, _, status| {
                                status.on_success(|w| GameState::Shop.proceed_to_target(w))
                            });
                        }
                    },
                );
                Tile::left("Arena Daily").title().close_btn().show_data(
                    &mut ws.arena_daily,
                    ctx,
                    |_, ui| {
                        br(ui);
                        if Button::click("Start new".into()).ui(ui).clicked() {
                            run_start_daily();
                            once_on_run_start_daily(|_, _, status| {
                                status.on_success(|w| GameState::Shop.proceed_to_target(w))
                            });
                        }
                    },
                );
                Tile::right("Normal Leaderboard").show_data(&mut ws.arena_normal, ctx, |d, ui| {
                    d.show_table("Normal Leaderboard", ui, world);
                });
                Tile::right("Daily Leaderboard").show_data(&mut ws.arena_daily, ctx, |d, ui| {
                    d.show_table("Daily Leaderboard", ui, world);
                });
                Tile::left("Settings").title().close_btn().show_data(
                    &mut ws.settings,
                    ctx,
                    |_, ui| {
                        let mut cs = client_settings().clone();
                        let vsync = if cs.vsync { "Enabled" } else { "Disabled" }.to_owned();
                        if Button::click(vsync)
                            .title("Vsync".into())
                            .set_bg(cs.vsync, ui)
                            .ui(ui)
                            .clicked()
                        {
                            cs.vsync = !cs.vsync;
                        }

                        if !cs.eq(&client_settings()) {
                            cs.save_to_file().apply(world);
                        }
                    },
                );
                Tile::left("Profile")
                    .close_btn()
                    .show_data(&mut ws.profile, ctx, |d, ui| {
                        ProfilePlugin::settings_ui(d, ui, world);
                    });

                Tile::left("Main Menu").title().open().show(ctx, |ui| {
                    format!("Welcome, {}!", LoginOption::get(world).user.name)
                        .cstr_cs(VISIBLE_LIGHT, CstrStyle::Heading2)
                        .label(ui);
                    br(ui);
                    let run = TArenaRun::get_current();
                    if let Some(run) = run.as_ref() {
                        let round = run.round;
                        let mode = match &run.mode {
                            GameMode::ArenaNormal => "Normal".to_owned(),
                            GameMode::ArenaConst(seed) => format!("Const ({seed})"),
                        };

                        if Button::click("Continue".into())
                            .title(format!("{mode} round {round}"))
                            .ui(ui)
                            .clicked()
                        {
                            GameState::Shop.proceed_to_target(world);
                        }
                        if Button::click("Abandon run".into()).red(ui).ui(ui).clicked() {
                            run_finish();
                        }
                        br(ui);
                    }
                    if Button::click("Arena Normal".into())
                        .enable_ui_with(
                            &mut ws.arena_normal,
                            || {
                                TArenaLeaderboard::iter()
                                    .filter(|d| d.mode.eq(&GameMode::ArenaNormal))
                                    .sorted_by_key(|d| -(d.round as i32))
                                    .collect_vec()
                            },
                            ui,
                        )
                        .clicked()
                    {
                        ws.arena_daily = None;
                    }
                    if Button::click("Arena Constant".into())
                        .enable_ui_with(
                            &mut ws.arena_daily,
                            || {
                                TArenaLeaderboard::iter()
                                    .filter(|d| {
                                        d.mode.eq(&GameMode::ArenaConst(
                                            chrono::Utc::now().date_naive().to_string(),
                                        ))
                                    })
                                    .sorted_by_key(|d| -(d.round as i32))
                                    .collect_vec()
                            },
                            ui,
                        )
                        .clicked()
                    {
                        ws.arena_normal = None;
                    }
                    br(ui);
                    Button::click("Settings".into()).enable_ui(&mut ws.settings, ui);
                    Button::click("Profile".into()).enable_ui(&mut ws.profile, ui);
                });
            }
            GameState::Battle => BattlePlugin::show_tiles(ctx, world),
            GameState::TableView(query) => TableViewPlugin::ui(query, ctx, world),

            _ => {}
        }
        let mut wd = world.remove_resource::<WidgetData>().unwrap();

        // Content
        CentralPanel::default()
            .frame(Frame::none())
            .show(ctx, |ui| match state {
                GameState::Shop => ShopPlugin::show_containers(&mut wd, ui, world),
                GameState::Connect => ConnectPlugin::ui(ui),
                GameState::Login => LoginPlugin::login_ui(ui, world),
                GameState::Battle => BattlePlugin::ui(ui, world),
                GameState::GameOver => ShopPlugin::game_over_ui(ui),
                GameState::TableView(query) => {
                    TableViewPlugin::ui_content(query, &mut wd, ui, world)
                }
                _ => {}
            });

        // Overlay
        match state {
            GameState::Shop => ShopPlugin::overlay_widgets(ctx, world),
            GameState::Battle => BattlePlugin::overlay_widgets(ctx, world),
            _ => {}
        }
        Notification::show_all(&wd, ctx, world);
        world.insert_resource(wd);
        world.insert_resource(ws);
    }
}
