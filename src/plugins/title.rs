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
                    .with_id("Settings".into())
                    .push(world);
                }
            });
        })
        .min_space(egui::vec2(200.0, 0.0))
        .pinned()
        .push(world);
        Tile::new(Side::Bottom, |ui, _| {
            ui.horizontal_centered(|ui| {
                if Button::click("Discord".into())
                    .icon(Icon::Discord)
                    .ui(ui)
                    .clicked()
                {
                    ui.ctx().open_url(egui::OpenUrl {
                        url: "https://discord.gg/c3UT58M9wb".into(),
                        new_tab: false,
                    });
                }
                if Button::click("Youtube".into())
                    .icon(Icon::Youtube)
                    .ui(ui)
                    .clicked()
                {
                    ui.ctx().open_url(egui::OpenUrl {
                        url: "https://www.youtube.com/@makscee".into(),
                        new_tab: false,
                    });
                }
                if Button::click("Github".into())
                    .icon(Icon::Github)
                    .ui(ui)
                    .clicked()
                {
                    ui.ctx().open_url(egui::OpenUrl {
                        url: "https://github.com/makscee/arena-of-ideas/releases".into(),
                        new_tab: false,
                    });
                }
            });
        })
        .pinned()
        .transparent()
        .no_expand()
        .push(world);
    }
}
