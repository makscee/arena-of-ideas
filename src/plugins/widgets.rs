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
    settings: Option<()>,
    profile: Option<ProfileEditData>,
}

impl WidgetsPlugin {
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
                    if Button::click("Start new".into()).ui(ui).clicked() {
                        run_start();
                        once_on_run_start(|_, _, status| {
                            status.on_success(|w| GameState::Shop.proceed_to_target(w))
                        });
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
                _ => {}
            });

        // Overlay
        match state {
            GameState::Shop => ShopPlugin::overlay_widgets(ctx, world),
            _ => {}
        }
        Notification::show_all(&wd, ctx, world);
        world.insert_resource(wd);
        world.insert_resource(ws);
    }
}
