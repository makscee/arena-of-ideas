use super::*;

pub struct SettingsPlugin;

impl SettingsPlugin {
    pub fn add_tile_video(world: &mut World) {
        Tile::new(Side::Left, |ui, world| {
            title("Video Settings", ui);
            let mut cs = client_settings().clone();
            let vsync = if cs.vsync { "Enabled" } else { "Disabled" }.to_owned();
            if Button::click(vsync)
                .title("Vsync".cstr())
                .set_bg(cs.vsync, ui)
                .ui(ui)
                .clicked()
            {
                cs.vsync = !cs.vsync;
            }

            if !cs.eq(&client_settings()) {
                cs.save_to_file().apply(world);
            }
        })
        .with_id("Video Settings".into())
        .min_space(egui::vec2(200.0, 0.0))
        .push(world);
    }
    pub fn add_tile_audio(world: &mut World) {
        Tile::new(Side::Left, |ui, world| {
            title("Audio Settings", ui);
            let mut cs = client_settings().clone();
            Slider::new("Master").ui(&mut cs.volume_master, 0.0..=1.0, ui);
            Slider::new("Music").ui(&mut cs.volume_music, 0.0..=1.0, ui);
            Slider::new("Sfx").ui(&mut cs.volume_fx, 0.0..=1.0, ui);

            if !cs.eq(&client_settings()) {
                cs.save_to_file().apply(world);
            }
        })
        .with_id("Audio Settings".into())
        .min_space(egui::vec2(200.0, 0.0))
        .push(world);
    }
}
