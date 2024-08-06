use super::*;

pub struct InboxPlugin;

impl InboxPlugin {
    pub fn ui_tiles(wd: &WidgetData, ctx: &egui::Context, world: &mut World) {
        Tile::left("Notifications Table")
            .open()
            .title()
            .show(ctx, |ui| Notification::show_all_table(wd, ui, world));
    }
}
