use super::*;
pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, Self::ui.run_if(in_state(GameState::MainMenu)));
    }
}

impl MainMenuPlugin {
    fn ui(world: &mut World) {
        let ctx = &egui_context(world);

        window("MAIN MENU")
            .set_width(400.0)
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                if LoginPlugin::is_connected() {
                    if CURRENT_USER.lock().unwrap().is_none() {
                        LoginPlugin::login(ui, world);
                        LoginPlugin::register(ui, world);
                    }
                } else {
                    ui.label("DISCONNECTED");
                    if ui.button("CONNECT").clicked() {
                        LoginPlugin::connect();
                    }
                }

                if LoginPlugin::get_username().is_some() {
                    frame(ui, |ui| {
                        let enabled = Save::get(world).is_ok();
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
                            Save {
                                mode: GameMode::GlobalTower,
                                climb: TowerClimb {
                                    shop: ShopState::new(world),
                                    team: default(),
                                    owner_team: default(),
                                    defeated: default(),
                                },
                            }
                            .save(world)
                            .unwrap();
                            GameState::change(GameState::Shop, world);
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
    }
}
