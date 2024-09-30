use super::*;

pub struct TitlePlugin;

impl TitlePlugin {
    pub fn add_tiles(world: &mut World) {
        Tile::new(Side::Left, |ui, world| {
            ui.vertical_centered_justified(|ui| {
                if Button::click("Play".into()).ui(ui).clicked() {
                    GameState::GameStart.proceed_to_target(world);
                }
                if Button::click("Settings".into()).ui(ui).clicked() {
                    Tile::new(Side::Left, |ui, world| {
                        title("Settings", ui);
                        ui.vertical_centered_justified(|ui| {
                            if Button::click("Video".into()).ui(ui).clicked() {
                                SettingsPlugin::add_tile_video(world);
                            }
                            if Button::click("Audio".into()).ui(ui).clicked() {
                                SettingsPlugin::add_tile_audio(world);
                            }
                            if Button::click("Profile".into()).ui(ui).clicked() {
                                ProfilePlugin::add_tile_settings(world);
                            }
                        });
                    })
                    .min_space(egui::vec2(200.0, 0.0))
                    .set_id("Settings".into())
                    .push(world);
                }
            });
        })
        .min_space(egui::vec2(200.0, 0.0))
        .pinned()
        .push(world);
    }
}
