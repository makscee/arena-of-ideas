use rand::seq::IteratorRandom;

use crate::module_bindings::start_new_ladder;

use super::*;
pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, Self::ui.run_if(in_state(GameState::MainMenu)))
            .add_systems(OnEnter(GameState::MainMenu), Self::on_enter)
            .add_systems(OnEnter(GameState::MainMenuClean), Self::on_enter_clean);
    }
}

impl MainMenuPlugin {
    fn on_enter_clean(world: &mut World) {
        let mut sd = SettingsData::load(world);
        sd.last_state_on_load = false;
        sd.save(world).unwrap();
        GameState::change(GameState::MainMenu, world);
    }

    fn on_enter(world: &mut World) {
        if SettingsData::get(world).last_state_on_load {
            if let Some(state) = PersistentData::load(world).last_state {
                GameState::change(state, world);
            }
        }
    }

    pub fn ui(world: &mut World) {
        let ctx = &egui_context(world);
        let save = Save::get(world);

        window("MAIN MENU").set_width(400.0)
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            frame(ui, |ui| {
                let enabled = save.current_level > 0;
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
                ui.columns(2, |ui| {
                    ui[0].vertical_centered_justified(|ui| {
                        if ui.button("RANDOM LADDDER").on_hover_text("Play ladder that belongs to other random player").clicked() {
                            if let Some(ladder) = TableLadder::iter()
                                .filter(|l| !l.status.eq(&module_bindings::LadderStatus::Building))
                                .choose(&mut thread_rng())
                            {
                                let save = Save {
                                    mode: GameMode::RandomLadder {
                                        ladder_id: ladder.id,
                                    },
                                    ..default()
                                };
                                save.save(world).unwrap();
                                GameState::change(GameState::Shop, world);
                            }
                        }
                    });
                    ui[1].vertical_centered_justified(|ui| {
                        if ui
                            .button("NEW LADDDER")
                            .on_hover_text(
                                "Generate new levels infinitely until defeat. New levels generated considering your teams strength",
                            )
                            .clicked()
                        {
                            start_new_ladder();
                            let save = Save { mode: GameMode::NewLadder, ..default() };
                            save.save(world).unwrap();
                            GameState::change(GameState::Shop, world);
                        }
                    });
                });
            });

            frame(ui, |ui| {
                if ui.button("PROFILE").clicked() {
                    GameState::change(GameState::Profile, world);
                }
            });
            frame(ui, |ui| {
                if ui.button("HERO GALLERY").clicked() {
                    GameState::change(GameState::HeroGallery, world);
                }
            });
            frame(ui, |ui| {
                if ui.button("RESET").clicked() {
                    Save::default().save(world).unwrap();
                    PersistentData::default().save(world).unwrap();
                    SettingsData::default().save(world).unwrap();
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
