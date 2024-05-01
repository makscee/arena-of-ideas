use bevy_egui::egui::Sense;

use crate::module_bindings::{once_on_run_start, run_start, ArenaRun};

use super::*;
pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            Self::ui
                .after(PanelsPlugin::ui)
                .run_if(in_state(GameState::MainMenu)),
        );
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
            .anchor(Align2::LEFT_TOP, [15.0, 200.0])
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
                        let run = ArenaRun::current();
                        let enabled = run.is_some();
                        ui.set_enabled(enabled);
                        let btn = if enabled {
                            ui.button_primary(format!("CONTINUE ({})", run.unwrap().round))
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
                if SettingsData::get(world).dev_mode {
                    frame(ui, |ui| {
                        if ui.button("CLIPBOARD BATTLE").clicked() {
                            GameState::change(GameState::ClipboardBattle, world);
                        }
                    });
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
        window("CONTACTS")
            .anchor(Align2::RIGHT_BOTTOM, [-15.0, -15.0])
            .set_width(75.0)
            .show(ctx, |ui| {
                Frame::none()
                    .inner_margin(Margin::same(8.0))
                    .show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            let mut resp = Icon::Discord.image().sense(Sense::click()).ui(ui);
                            resp = resp.on_hover_text("Discord server");
                            if resp.hovered() {
                                Icon::Discord.image().tint(yellow()).paint_at(ui, resp.rect);
                            }
                            if resp.clicked() {
                                ctx.open_url(egui::OpenUrl {
                                    url: "https://discord.com/invite/YxjYxc3ZXP".to_owned(),
                                    new_tab: true,
                                });
                            }
                            let mut resp = Icon::Youtube.image().sense(Sense::click()).ui(ui);
                            resp = resp.on_hover_text("Youtube channel");
                            if resp.hovered() {
                                Icon::Youtube.image().tint(yellow()).paint_at(ui, resp.rect);
                            }
                            if resp.clicked() {
                                ctx.open_url(egui::OpenUrl {
                                    url: "https://www.youtube.com/@makscee".to_owned(),
                                    new_tab: true,
                                });
                            }
                            let mut resp = Icon::Itch.image().sense(Sense::click()).ui(ui);
                            resp = resp.on_hover_text("Itch.io page");
                            if resp.hovered() {
                                Icon::Itch.image().tint(yellow()).paint_at(ui, resp.rect);
                            }
                            if resp.clicked() {
                                ctx.open_url(egui::OpenUrl {
                                    url: "https://makscee.itch.io/arena-of-ideas".to_owned(),
                                    new_tab: true,
                                });
                            }
                            let mut resp = Icon::Github.image().sense(Sense::click()).ui(ui);
                            resp = resp.on_hover_text("Github page");
                            if resp.hovered() {
                                Icon::Github.image().tint(yellow()).paint_at(ui, resp.rect);
                            }
                            if resp.clicked() {
                                ctx.open_url(egui::OpenUrl {
                                    url: "https://github.com/makscee/arena-of-ideas".to_owned(),
                                    new_tab: true,
                                });
                            }
                        });
                    });
            });
        LeaderboardPlugin::ui(world);
    }
}
