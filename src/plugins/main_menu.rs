use bevy_egui::egui::Sense;

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
        if let Ok(camera) = world
            .query_filtered::<Entity, With<Camera>>()
            .get_single(world)
        {
            world.entity_mut(camera).despawn_recursive();
        }
        if SettingsData::get(world).last_state_on_load {
            if let Some(state) = PersistentData::load(world).last_state {
                GameState::change(state, world);
            }
        }
    }

    pub fn ui(world: &mut World) {
        let ctx = &egui_context(world);
        let mut save = Save::get(world).unwrap();
        Window::new("Menu")
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .resizable(false)
            .title_bar(false)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    let btn = Self::menu_button("Continue".to_owned(), ui);
                    let can_continue = save.current_level > 0;
                    if ui.add_enabled(can_continue, btn).clicked() {
                        GameState::change(GameState::Shop, world);
                    }
                    let btn = Self::menu_button("New Ladder".to_owned(), ui);
                    if ui.add(btn).clicked() {
                        Save::default().save(world).unwrap();
                        GameState::change(GameState::Shop, world);
                    }
                    let has_old = !save.ladder.teams.is_empty();
                    let btn = Self::menu_button("Old Ladder".to_owned(), ui);
                    if ui.add_enabled(has_old, btn).clicked() {
                        save.current_level = 0;
                        save.team = default();
                        save.save(world).unwrap();
                        GameState::change(GameState::Shop, world);
                    }
                    let btn = Self::menu_button("Custom Battle".to_owned(), ui);
                    if ui.add(btn).clicked() {
                        GameState::change(GameState::CustomBattle, world);
                    }
                    let btn = Self::menu_button("Hero Gallery".to_owned(), ui);
                    if ui.add(btn).clicked() {
                        GameState::change(GameState::HeroGallery, world);
                    }
                    let btn = Self::menu_button("Hero Editor".to_owned(), ui);
                    if ui.add(btn).clicked() {
                        GameState::change(GameState::HeroEditor, world);
                    }
                    let btn = Self::menu_button("Run Tests".to_owned(), ui);
                    if ui.add(btn).clicked() {
                        GameState::change(GameState::TestsLoading, world);
                    }
                    ui.add_space(15.0);
                });
            });
    }

    fn menu_button(name: String, ui: &mut Ui) -> Button {
        ui.add_space(15.0);
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
