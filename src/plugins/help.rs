use bevy_egui::egui::ScrollArea;
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};

use super::*;

pub struct HelpPlugin;

impl Plugin for HelpPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(HelpData::default());
    }
}

#[derive(Resource, Default)]
struct HelpData {
    cache: CommonMarkCache,
}

impl HelpPlugin {
    pub fn ui(world: &mut World) {
        let ctx = &if let Some(context) = egui_context(world) {
            context
        } else {
            return;
        };
        let cache = &mut world.resource_mut::<HelpData>().cache;
        let text = include_str!("../../assets/md/help.md");
        window("HELP")
            .set_width(400.0)
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                ScrollArea::new([false, true]).show(ui, |ui| {
                    frame(ui, |ui| {
                        CommonMarkViewer::new("help")
                            .default_width(Some(600))
                            .show(ui, cache, text);
                    });
                });
            });
    }
}
