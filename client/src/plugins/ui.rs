use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin, egui};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default())
            .init_resource::<FontsLoaded>()
            .add_systems(bevy_egui::EguiPrimaryContextPass, setup_fonts);
    }
}

#[derive(Resource, Default)]
struct FontsLoaded(bool);

fn setup_fonts(mut contexts: EguiContexts, mut loaded: ResMut<FontsLoaded>) {
    if loaded.0 {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else { return };

    let mut fonts = egui::FontDefinitions::default();

    // Load SometypeMono as main font
    if let Ok(font_data) = std::fs::read("assets/fonts/SometypeMono-Regular.ttf") {
        fonts.font_data.insert(
            "SometypeMono".to_owned(),
            std::sync::Arc::new(egui::FontData::from_owned(font_data)),
        );
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "SometypeMono".to_owned());
        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .insert(0, "SometypeMono".to_owned());
    }

    // Load NotoEmoji as fallback for emoji/symbol characters
    if let Ok(font_data) = std::fs::read("assets/fonts/NotoEmoji-VariableFont_wght.ttf") {
        fonts.font_data.insert(
            "NotoEmoji".to_owned(),
            std::sync::Arc::new(egui::FontData::from_owned(font_data)),
        );
        // Add as last fallback — only used when main fonts don't have the glyph
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .push("NotoEmoji".to_owned());
    }

    ctx.set_fonts(fonts);
    loaded.0 = true;
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

pub fn tier_color(tier: u8) -> bevy_egui::egui::Color32 {
    colors::TIER_COLORS
        .get(tier.saturating_sub(1) as usize)
        .copied()
        .unwrap_or(bevy_egui::egui::Color32::WHITE)
}

pub fn rating_color(rating: i32) -> bevy_egui::egui::Color32 {
    if rating > 0 {
        colors::RATING_POSITIVE
    } else if rating < 0 {
        colors::RATING_NEGATIVE
    } else {
        bevy_egui::egui::Color32::GRAY
    }
}
