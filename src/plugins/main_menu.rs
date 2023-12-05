use bevy_egui::egui::InnerResponse;

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
            let btn = Self::menu_button("New Ladder");
            let text = r#"Generate new levels infinitely until defeat.\
            New levels generated considering your teams strength"#;

            if ui.add(btn).on_hover_text(text).clicked() {
                Save::default().save(world).unwrap();
                GameState::change(GameState::Shop, world);
            }
            let has_old = !save.ladder.teams.is_empty();
            let name = if has_old {
                format!(
                    "Old Ladder: {} levels",
                    save.ladder.teams.len() + Options::get_initial_ladder(world).teams.len()
                )
            } else {
                "Old Ladder".to_owned()
            };
            let btn = Self::menu_button(&name);
            if ui
                .add_enabled(has_old, btn)
                .on_hover_text("Play last ladder starting from level 1")
                .on_disabled_hover_text("Play new ladder, then continue it after defeat")
                .clicked()
            {
                save.current_level = 0;
                save.team = default();
                save.save(world).unwrap();
                GameState::change(GameState::Shop, world);
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
