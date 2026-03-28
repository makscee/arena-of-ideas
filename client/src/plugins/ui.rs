use bevy::prelude::*;
use bevy_egui::EguiPlugin;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default());
    }
}

/// Standard colors used throughout the UI.
pub mod colors {
    use bevy_egui::egui;

    pub const ABILITY_COLOR: egui::Color32 = egui::Color32::from_rgb(100, 180, 255);
    pub const UNIT_COLOR: egui::Color32 = egui::Color32::from_rgb(255, 180, 100);
    pub const TIER_COLORS: [egui::Color32; 5] = [
        egui::Color32::from_rgb(180, 180, 180), // Tier 1 - grey
        egui::Color32::from_rgb(100, 200, 100), // Tier 2 - green
        egui::Color32::from_rgb(100, 150, 255), // Tier 3 - blue
        egui::Color32::from_rgb(200, 100, 255), // Tier 4 - purple
        egui::Color32::from_rgb(255, 200, 50),  // Tier 5 - gold
    ];
    pub const RATING_POSITIVE: egui::Color32 = egui::Color32::from_rgb(100, 255, 100);
    pub const RATING_NEGATIVE: egui::Color32 = egui::Color32::from_rgb(255, 100, 100);
}

/// Helper to get tier color by tier number (1-5).
pub fn tier_color(tier: u8) -> bevy_egui::egui::Color32 {
    colors::TIER_COLORS
        .get(tier.saturating_sub(1) as usize)
        .copied()
        .unwrap_or(bevy_egui::egui::Color32::WHITE)
}

/// Helper to format rating with color.
pub fn rating_color(rating: i32) -> bevy_egui::egui::Color32 {
    if rating > 0 {
        colors::RATING_POSITIVE
    } else if rating < 0 {
        colors::RATING_NEGATIVE
    } else {
        bevy_egui::egui::Color32::GRAY
    }
}
