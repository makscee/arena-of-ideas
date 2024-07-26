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
                Tile::right("Arena Normal").title().close_btn().show_data(
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
                Tile::right("Arena Daily").title().close_btn().show_data(
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
                Tile::left("Normal Leaderboard").show_data(&mut ws.arena_normal, ctx, |d, ui| {
                    d.show_table("Normal Leaderboard", ui, world);
                });
                Tile::left("Daily Leaderboard").show_data(&mut ws.arena_daily, ctx, |d, ui| {
                    d.show_table("Daily Leaderboard", ui, world);
                });
                Tile::right("Settings").title().close_btn().show_data(
                    &mut ws.settings,
                    ctx,
                    |_, ui| {
                        if ui.button("setting 1").clicked() {
                            debug!("Test click");
                        }
                        br(ui);
                        if ui.button("setting 2").clicked() {
                            debug!("Test click");
                        }
                        br(ui);
                    },
                );
                Tile::right("Profile")
                    .close_btn()
                    .show_data(&mut ws.profile, ctx, |d, ui| {
                        ProfilePlugin::settings_ui(d, ui, world);
                    });

                Tile::right("Main Menu").title().open().show(ctx, |ui| {
                    format!("Welcome, {}!", LoginOption::get(world).user.name)
                        .cstr_cs(VISIBLE_LIGHT, CstrStyle::Heading2)
                        .label(ui);
                    br(ui);
                    let run = TArenaRun::get_current();
                    if Button::click("Continue".into())
                        .enabled(run.is_some())
                        .ui(ui)
                        .clicked()
                    {
                        GameState::Shop.proceed_to_target(world);
                    }
                    br(ui);
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
                    if Button::click("Arena Daily".into())
                        .enable_ui_with(
                            &mut ws.arena_daily,
                            || {
                                TArenaLeaderboard::iter()
                                    .filter(|d| {
                                        d.mode.eq(&GameMode::ArenaDaily(
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
