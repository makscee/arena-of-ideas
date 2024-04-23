use crate::module_bindings::{once_on_run_start, run_start, ArenaRun};

use super::*;
pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, Self::ui.run_if(in_state(GameState::MainMenu)));
    }
}

impl MainMenuPlugin {
    fn ui(world: &mut World) {
        let ctx = &if let Some(context) = egui_context(world) {
            context
        } else {
            return;
        };

        window("MAIN MENU")
            .set_width(400.0)
            .anchor(Align2::LEFT_TOP, [20.0, 200.0])
            .show(ctx, |ui| {
                if LoginPlugin::is_connected() {
                    if LoginPlugin::get_user_data().is_none() {
                        LoginPlugin::login(ui, world);
                    }
                } else {
                    frame(ui, |ui| {
                        ui.label("DISCONNECTED");
                        if ui.button("CONNECT").clicked() {
                            LoginPlugin::connect(world);
                        }
                    });
                }

                if let Some(user) = LoginPlugin::get_user_data() {
                    let name = user.name;
                    frame(ui, |ui| {
                        ui.label(format!("Welcome {name}!"));
                    });
                    frame(ui, |ui| {
                        let enabled = ArenaRun::current().is_some();
                        ui.set_enabled(enabled);
                        let btn = if enabled {
                            ui.button_primary("CONTINUE")
                        } else {
                            ui.button("CONTINUE")
                        };
                        if btn.clicked() {
                            GameState::change(GameState::Shop, world);
                        }
                    });
                    frame(ui, |ui| {
                        if ui.button("NEW GAME").clicked() {
                            debug!("Start new run");
                            run_start();
                            once_on_run_start(|_, _, s| {
                                debug!("Run start callback: {s:?}");
                                match s {
                                    spacetimedb_sdk::reducer::Status::Committed => {
                                        OperationsPlugin::add(|w| {
                                            GameState::change(GameState::Shop, w);
                                        })
                                    }
                                    spacetimedb_sdk::reducer::Status::Failed(e) => {
                                        AlertPlugin::add_error(
                                            Some("GAME START ERROR".to_owned()),
                                            e.clone(),
                                            None,
                                        )
                                    }
                                    _ => panic!(),
                                };
                            });
                        }
                    });
                }

                frame(ui, |ui| {
                    if ui.button("HERO GALLERY").clicked() {
                        GameState::change(GameState::HeroGallery, world);
                    }
                });
                if cfg!(debug_assertions) {
                    frame(ui, |ui| {
                        ui.columns(3, |ui| {
                            ui[0].vertical_centered_justified(|ui| {
                                if ui.button("CUSTOM").clicked() {
                                    GameState::change(GameState::CustomBattle, world);
                                }
                            });
                            ui[1].vertical_centered_justified(|ui| {
                                if ui.button("EDITOR").clicked() {
                                    GameState::change(GameState::HeroEditor, world);
                                }
                            });
                            ui[2].vertical_centered_justified(|ui| {
                                if ui.button("TESTS").clicked() {
                                    GameState::change(GameState::TestsLoading, world);
                                }
                            });
                        })
                    });
                }
            });
        LeaderboardPlugin::ui(world);
    }
}
