use bevy_egui::egui::InnerResponse;
use rand::seq::IteratorRandom;

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
        let mut save = Save::get(world);
        Self::menu_window("Menu", ctx, |ui| {
            let can_continue = save.current_level > 0;
            if ui
                .add_enabled(
                    can_continue,
                    Button::new(
                        RichText::new("Continue")
                            .size(20.0)
                            .text_style(egui::TextStyle::Heading)
                            .color(hex_color!("#ffffff")),
                    )
                    .min_size(egui::vec2(200.0, 0.0)),
                )
                .on_hover_text("Continue last game")
                .clicked()
            {
                GameState::change(GameState::Shop, world);
            }
            let btn = Self::menu_button("Random Ladder");
            let text = r#"Play ladder that belongs to other random player"#;
            if ui.add(btn).on_hover_text(text).clicked() {
                if let Some(ladder) = LadderTable::iter().choose(&mut thread_rng()) {
                    let mut save = Save::get(world);
                    save.current_level = 0;
                    save.ladder.levels = ladder.levels;
                    save.save(world).unwrap();

                    GameState::change(GameState::Shop, world);
                }
            }
            let btn = Self::menu_button("New Ladder");
            let text = r#"Generate new levels infinitely until defeat.
New levels generated considering your teams strength"#;

            if ui.add(btn).on_hover_text(text).clicked() {
                Save::default().save(world).unwrap();
                GameState::change(GameState::Shop, world);
            }
            if let Ok(identity) = identity() {
                let (enabled, name, levels) = if let Ok(ladder) =
                    LadderTable::filter_by_owner(identity).exactly_one()
                {
                    (
                        true,
                        format!(
                            "My Best Ladder ({})",
                            ladder.levels.len() + Options::get_initial_ladder(world).levels.len()
                        ),
                        ladder.levels.clone(),
                    )
                } else {
                    (false, "My Best Ladder".to_owned(), default())
                };
                let btn = Self::menu_button(&name);
                if ui
                    .add_enabled(enabled, btn)
                    .on_hover_text("Play own longest ladder starting from level 1")
                    .clicked()
                {
                    save.current_level = 0;
                    save.team = default();
                    save.ladder.levels = levels;
                    save.save(world).unwrap();
                    GameState::change(GameState::Shop, world);
                }
            }
            let btn = Self::menu_button("Custom Battle");
            if ui.add(btn).clicked() {
                GameState::change(GameState::CustomBattle, world);
            }
            let btn = Self::menu_button("Hero Gallery");
            if ui.add(btn).clicked() {
                GameState::change(GameState::HeroGallery, world);
            }
            let btn = Self::menu_button("Hero Editor");
            if ui.add(btn).clicked() {
                GameState::change(GameState::HeroEditor, world);
            }
            let btn = Self::menu_button("Run Tests");
            if ui.add(btn).clicked() {
                GameState::change(GameState::TestsLoading, world);
            }
            let btn = Self::menu_button("Profile");
            if ui.add(btn).clicked() {
                GameState::change(GameState::Profile, world);
            }
            let btn = Self::menu_button("Reset");
            if ui.add(btn).clicked() {
                Save::default().save(world).unwrap();
                PersistentData::default().save(world).unwrap();
                SettingsData::default().save(world).unwrap();
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
