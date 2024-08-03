use bevy::{ecs::schedule::Condition, input::common_conditions::input_just_pressed};
use egui::Area;

use super::*;

pub struct WidgetsPlugin;

impl Plugin for WidgetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, Self::ui)
            .init_resource::<WidgetsState>();

        if cfg!(debug_assertions) {
            app.add_systems(
                Update,
                give_c
                    .run_if(input_just_pressed(KeyCode::KeyG).and_then(in_state(GameState::Title))),
            );
        }
    }
}

fn give_c() {
    give_credits();
}

#[derive(Default, Resource)]
struct WidgetsState {
    arena_normal: Option<Vec<TArenaLeaderboard>>,
    arena_ranked: Option<Vec<TArenaLeaderboard>>,
    arena_const: Option<Vec<TArenaLeaderboard>>,
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
                Tile::left("Main Menu")
                    .title()
                    .default_size(300.0)
                    .open()
                    .show(ctx, |ui| {
                        text_dots_text(&"name".cstr(), &user_name().cstr_c(VISIBLE_LIGHT), ui);
                        text_dots_text(
                            &"credits".cstr(),
                            &TWallet::current().amount.to_string().cstr_c(VISIBLE_LIGHT),
                            ui,
                        );
                        space(ui);
                        let run = TArenaRun::get_current();
                        if let Some(run) = run.as_ref() {
                            let round = run.round;
                            let txt = run
                                .mode
                                .cstr()
                                .push(" round ".cstr())
                                .push(round.to_string().cstr_c(VISIBLE_BRIGHT))
                                .style(CstrStyle::Small)
                                .take();

                            if Button::click("Continue".into()).title(txt).ui(ui).clicked() {
                                GameState::Shop.proceed_to_target(world);
                            }
                            if Button::click("Abandon run".into()).red(ui).ui(ui).clicked() {
                                Confirmation::new(
                                    "Abandon current run?".cstr_c(VISIBLE_BRIGHT),
                                    |_| {
                                        run_finish();
                                    },
                                )
                                .add(ui.ctx());
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
                            ws.arena_const = None;
                            ws.arena_ranked = None;
                        }
                        if Button::click("Arena Ranked".into())
                            .color(YELLOW, ui)
                            .enable_ui_with(
                                &mut ws.arena_ranked,
                                || {
                                    TArenaLeaderboard::iter()
                                        .filter(|d| d.mode.eq(&GameMode::ArenaRanked))
                                        .sorted_by_key(|d| -(d.round as i32))
                                        .collect_vec()
                                },
                                ui,
                            )
                            .clicked()
                        {
                            ws.arena_const = None;
                            ws.arena_normal = None;
                        }
                        if Button::click("Arena Constant".into())
                            .color(CYAN, ui)
                            .enable_ui_with(
                                &mut ws.arena_const,
                                || {
                                    TArenaLeaderboard::iter()
                                        .filter(|d| {
                                            d.mode.eq(&GameMode::ArenaConst(
                                                GlobalData::current().constant_seed,
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
                            ws.arena_ranked = None;
                        }
                        br(ui);
                        Button::click("Settings".into()).enable_ui(&mut ws.settings, ui);
                        Button::click("Profile".into()).enable_ui(&mut ws.profile, ui);
                        br(ui);
                        if Button::click("Exit".into()).gray(ui).ui(ui).clicked() {
                            Confirmation::new("Exit the game?".cstr_c(VISIBLE_BRIGHT), app_exit)
                                .add(ctx);
                        }
                    });

                Tile::left("Arena Normal").title().close_btn().show_data(
                    &mut ws.arena_normal,
                    ctx,
                    |_, ui| {
                        if Button::click("Start new".into()).ui(ui).clicked() {
                            run_start_normal();
                            once_on_run_start_normal(|_, _, status| {
                                status.on_success(|w| GameState::Shop.proceed_to_target(w))
                            });
                        }
                    },
                );
                Tile::left("Arena Ranked").title().close_btn().show_data(
                    &mut ws.arena_ranked,
                    ctx,
                    |_, ui| {
                        let cost = GameAssets::get(world).global_settings.arena.ranked_cost;
                        let can_afford = TWallet::current().amount >= cost;
                        if Button::click(format!("-{cost} C"))
                            .color(YELLOW, ui)
                            .title("Start new".cstr())
                            .enabled(can_afford)
                            .ui(ui)
                            .clicked()
                        {
                            run_start_ranked();
                            once_on_run_start_ranked(|_, _, status| {
                                status.on_success(|w| GameState::Shop.proceed_to_target(w))
                            });
                        }
                    },
                );
                Tile::left("Arena Const").title().close_btn().show_data(
                    &mut ws.arena_const,
                    ctx,
                    |_, ui| {
                        if Button::click("Start new".into()).ui(ui).clicked() {
                            run_start_const();
                            once_on_run_start_const(|_, _, status| {
                                status.on_success(|w| GameState::Shop.proceed_to_target(w))
                            });
                        }
                    },
                );
                Tile::right("Normal Leaderboard").show_data(&mut ws.arena_normal, ctx, |d, ui| {
                    d.show_table("Normal Leaderboard", ui, world);
                });
                Tile::right("Ranked Leaderboard").show_data(&mut ws.arena_ranked, ctx, |d, ui| {
                    d.show_table("Ranked Leaderboard", ui, world);
                });
                Tile::right("Const Leaderboard").show_data(&mut ws.arena_const, ctx, |d, ui| {
                    d.show_table("Const Leaderboard", ui, world);
                });
                Tile::left("Settings").title().close_btn().show_data(
                    &mut ws.settings,
                    ctx,
                    |_, ui| {
                        let mut cs = client_settings().clone();
                        let vsync = if cs.vsync { "Enabled" } else { "Disabled" }.to_owned();
                        if Button::click(vsync)
                            .title("Vsync".cstr())
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
        Confirmation::show_current(ctx, world);
        world.insert_resource(wd);
        world.insert_resource(ws);
    }
}
