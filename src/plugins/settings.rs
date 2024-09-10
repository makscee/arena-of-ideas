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
        .set_id("Video Settings".into())
        .push(world);
    }
}
