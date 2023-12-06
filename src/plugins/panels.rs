use bevy_egui::egui::{Layout, TopBottomPanel};

use super::*;

pub struct PanelsPlugin;

impl Plugin for PanelsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, Self::ui);
    }
}

impl PanelsPlugin {
    pub fn ui(world: &mut World) {
        TopBottomPanel::top("top").show(&egui_context(world), |ui| {
            ui.with_layout(Layout::right_to_left(egui::Align::Min), |ui| {
                ui.set_max_width(250.0);
                SettingsPlugin::ui(ui, world);
                ui.set_max_width(200.0);
                ProfilePlugin::ui(ui, world);
            })
        });
    }
}
