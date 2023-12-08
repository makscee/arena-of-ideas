use bevy_egui::egui::InnerResponse;
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

        let mut window = window("MAIN MENU");
        window.0 = window.0.anchor(Align2::CENTER_CENTER, [0.0, 0.0]);
        window.show(ctx, |ui| {
            frame(ui, |ui| {
                ui.set_enabled(save.current_level > 0);
                if ui.button("CONTINUE").clicked() {
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
                                let mut save = Save::default();
                                save.mode = GameMode::RandomLadder {
                                    ladder_id: ladder.id,
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
                            let mut save = Save::default();
                            save.mode = GameMode::NewLadder;
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
                            if ui.button("CUSTOM BATTLE").clicked() {
                                GameState::change(GameState::CustomBattle, world);
                            }
                        });
                        ui[1].vertical_centered_justified(|ui| {
                            if ui.button("HERO EDITOR").clicked() {
                                GameState::change(GameState::HeroEditor, world);
                            }
                        });
                        ui[2].vertical_centered_justified(|ui| {
                            if ui.button("RUN TESTS").clicked() {
                                GameState::change(GameState::TestsLoading, world);
                            }
                        });
                    })
                });
            }
        });
    }

    pub fn menu_window<R>(
        name: &str,
        ctx: &egui::Context,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> Option<InnerResponse<Option<()>>> {
        Window::new(name)
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .resizable(false)
            .collapsible(false)
            .show(ctx, |ui| {
                let s = ui.style_mut();
                s.spacing.item_spacing.y = 20.0;
                ui.add_space(15.0);
                ui.vertical_centered(add_contents);
                ui.add_space(0.0);
            })
    }

    pub fn menu_button(name: &str) -> Button {
        let btn = Button::new(
            RichText::new(name)
                .size(20.0)
                .text_style(egui::TextStyle::Heading)
                .color(hex_color!("#ffffff")),
        )
        .min_size(egui::vec2(200.0, 0.0));
        btn
    }
}
