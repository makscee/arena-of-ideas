use parking_lot::{Mutex, MutexGuard};
use serde::{Deserialize, Serialize};

use super::*;

// Main Colorix struct with arrays of raw and generated colors
#[derive(Resource, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Colorix {
    // Raw colors (serialized) - 12 color slots
    pub raw_colors: [egui::Color32; 12],
    dark_mode: bool,

    // Generated colors (not serialized)
    #[serde(skip)]
    colors: Option<[egui::Color32; 12]>,
}

impl Default for Colorix {
    fn default() -> Self {
        Self {
            raw_colors: [egui::Color32::GRAY; 12],
            dark_mode: true,
            colors: None,
        }
    }
}

impl Colorix {
    pub fn new(base_color: egui::Color32, dark_mode: bool) -> Self {
        let mut colorix = Self {
            raw_colors: [base_color; 12],
            dark_mode,
            colors: None,
        };
        colorix.generate_scale();
        colorix
    }

    pub fn from_rgb(rgb: [u8; 3], dark_mode: bool) -> Self {
        let base_color = egui::Color32::from_rgb(rgb[0], rgb[1], rgb[2]);
        Self::new(base_color, dark_mode)
    }

    // Individual color setters by index
    pub fn set_app_background(&mut self, color: egui::Color32) -> &mut Self {
        self.raw_colors[0] = color;
        self
    }

    pub fn set_subtle_background(&mut self, color: egui::Color32) -> &mut Self {
        self.raw_colors[1] = color;
        self
    }

    pub fn set_ui_element_background(&mut self, color: egui::Color32) -> &mut Self {
        self.raw_colors[2] = color;
        self
    }

    pub fn set_hovered_ui_element_background(&mut self, color: egui::Color32) -> &mut Self {
        self.raw_colors[3] = color;
        self
    }

    pub fn set_active_ui_element_background(&mut self, color: egui::Color32) -> &mut Self {
        self.raw_colors[4] = color;
        self
    }

    pub fn set_subtle_borders_and_separators(&mut self, color: egui::Color32) -> &mut Self {
        self.raw_colors[5] = color;
        self
    }

    pub fn set_ui_element_border_and_focus_rings(&mut self, color: egui::Color32) -> &mut Self {
        self.raw_colors[6] = color;
        self
    }

    pub fn set_hovered_ui_element_border(&mut self, color: egui::Color32) -> &mut Self {
        self.raw_colors[7] = color;
        self
    }

    pub fn set_solid_backgrounds(&mut self, color: egui::Color32) -> &mut Self {
        self.raw_colors[8] = color;
        self
    }

    pub fn set_hovered_solid_backgrounds(&mut self, color: egui::Color32) -> &mut Self {
        self.raw_colors[9] = color;
        self
    }

    pub fn set_low_contrast_text(&mut self, color: egui::Color32) -> &mut Self {
        self.raw_colors[10] = color;
        self
    }

    pub fn set_high_contrast_text(&mut self, color: egui::Color32) -> &mut Self {
        self.raw_colors[11] = color;
        self
    }

    // Generate color scale and store in colors array
    pub fn generate_scale(&mut self) -> &mut Self {
        let generated_colors = Self::generate_color_scale(self.raw_colors, self.dark_mode);
        self.colors = Some(generated_colors);
        self
    }

    // Generate 12-step color scale from array of raw colors
    fn generate_color_scale(
        raw_colors: [egui::Color32; 12],
        dark_mode: bool,
    ) -> [egui::Color32; 12] {
        let mut generated_colors = [egui::Color32::GRAY; 12];

        for i in 0..12 {
            let [r, g, b, _] = raw_colors[i].to_array();

            generated_colors[i] = match i {
                0 => {
                    // app_background
                    if dark_mode {
                        egui::Color32::from_rgb(
                            (r as f32 * 0.08) as u8,
                            (g as f32 * 0.08) as u8,
                            (b as f32 * 0.08) as u8,
                        )
                    } else {
                        egui::Color32::from_rgb(
                            255 - ((255 - r) as f32 * 0.05) as u8,
                            255 - ((255 - g) as f32 * 0.05) as u8,
                            255 - ((255 - b) as f32 * 0.05) as u8,
                        )
                    }
                }
                1 => {
                    // subtle_background
                    if dark_mode {
                        egui::Color32::from_rgb(
                            (r as f32 * 0.12) as u8,
                            (g as f32 * 0.12) as u8,
                            (b as f32 * 0.12) as u8,
                        )
                    } else {
                        egui::Color32::from_rgb(
                            255 - ((255 - r) as f32 * 0.08) as u8,
                            255 - ((255 - g) as f32 * 0.08) as u8,
                            255 - ((255 - b) as f32 * 0.08) as u8,
                        )
                    }
                }
                2 => {
                    // ui_element_background
                    if dark_mode {
                        egui::Color32::from_rgb(
                            (r as f32 * 0.18) as u8,
                            (g as f32 * 0.18) as u8,
                            (b as f32 * 0.18) as u8,
                        )
                    } else {
                        egui::Color32::from_rgb(
                            255 - ((255 - r) as f32 * 0.12) as u8,
                            255 - ((255 - g) as f32 * 0.12) as u8,
                            255 - ((255 - b) as f32 * 0.12) as u8,
                        )
                    }
                }
                3 => {
                    // hovered_ui_element_background
                    if dark_mode {
                        egui::Color32::from_rgb(
                            (r as f32 * 0.25) as u8,
                            (g as f32 * 0.25) as u8,
                            (b as f32 * 0.25) as u8,
                        )
                    } else {
                        egui::Color32::from_rgb(
                            255 - ((255 - r) as f32 * 0.18) as u8,
                            255 - ((255 - g) as f32 * 0.18) as u8,
                            255 - ((255 - b) as f32 * 0.18) as u8,
                        )
                    }
                }
                4 => {
                    // active_ui_element_background
                    if dark_mode {
                        egui::Color32::from_rgb(
                            (r as f32 * 0.35) as u8,
                            (g as f32 * 0.35) as u8,
                            (b as f32 * 0.35) as u8,
                        )
                    } else {
                        egui::Color32::from_rgb(
                            255 - ((255 - r) as f32 * 0.25) as u8,
                            255 - ((255 - g) as f32 * 0.25) as u8,
                            255 - ((255 - b) as f32 * 0.25) as u8,
                        )
                    }
                }
                5 => {
                    // subtle_borders_and_separators
                    if dark_mode {
                        egui::Color32::from_rgb(
                            (r as f32 * 0.45) as u8,
                            (g as f32 * 0.45) as u8,
                            (b as f32 * 0.45) as u8,
                        )
                    } else {
                        egui::Color32::from_rgb(
                            255 - ((255 - r) as f32 * 0.35) as u8,
                            255 - ((255 - g) as f32 * 0.35) as u8,
                            255 - ((255 - b) as f32 * 0.35) as u8,
                        )
                    }
                }
                6 => {
                    // ui_element_border_and_focus_rings
                    if dark_mode {
                        egui::Color32::from_rgb(
                            (r as f32 * 0.55) as u8,
                            (g as f32 * 0.55) as u8,
                            (b as f32 * 0.55) as u8,
                        )
                    } else {
                        egui::Color32::from_rgb(
                            255 - ((255 - r) as f32 * 0.45) as u8,
                            255 - ((255 - g) as f32 * 0.45) as u8,
                            255 - ((255 - b) as f32 * 0.45) as u8,
                        )
                    }
                }
                7 => {
                    // hovered_ui_element_border
                    if dark_mode {
                        egui::Color32::from_rgb(
                            (r as f32 * 0.65) as u8,
                            (g as f32 * 0.65) as u8,
                            (b as f32 * 0.65) as u8,
                        )
                    } else {
                        egui::Color32::from_rgb(
                            255 - ((255 - r) as f32 * 0.55) as u8,
                            255 - ((255 - g) as f32 * 0.55) as u8,
                            255 - ((255 - b) as f32 * 0.55) as u8,
                        )
                    }
                }
                8 => {
                    // solid_backgrounds
                    if dark_mode {
                        egui::Color32::from_rgb(
                            (r as f32 * 0.75) as u8,
                            (g as f32 * 0.75) as u8,
                            (b as f32 * 0.75) as u8,
                        )
                    } else {
                        egui::Color32::from_rgb(
                            (r as f32 * 0.7) as u8,
                            (g as f32 * 0.7) as u8,
                            (b as f32 * 0.7) as u8,
                        )
                    }
                }
                9 => {
                    // hovered_solid_backgrounds
                    if dark_mode {
                        egui::Color32::from_rgb(
                            (r as f32 * 0.85) as u8,
                            (g as f32 * 0.85) as u8,
                            (b as f32 * 0.85) as u8,
                        )
                    } else {
                        egui::Color32::from_rgb(
                            (r as f32 * 0.6) as u8,
                            (g as f32 * 0.6) as u8,
                            (b as f32 * 0.6) as u8,
                        )
                    }
                }
                10 => {
                    // low_contrast_text
                    if dark_mode {
                        egui::Color32::from_rgb(
                            255.min(r as u32 + 80) as u8,
                            255.min(g as u32 + 80) as u8,
                            255.min(b as u32 + 80) as u8,
                        )
                    } else {
                        egui::Color32::from_rgb(
                            (r as f32 * 0.5) as u8,
                            (g as f32 * 0.5) as u8,
                            (b as f32 * 0.5) as u8,
                        )
                    }
                }
                11 => {
                    // high_contrast_text
                    if dark_mode {
                        egui::Color32::from_rgb(
                            255.min(r as u32 + 120) as u8,
                            255.min(g as u32 + 120) as u8,
                            255.min(b as u32 + 120) as u8,
                        )
                    } else {
                        egui::Color32::from_rgb(
                            (r as f32 * 0.2) as u8,
                            (g as f32 * 0.2) as u8,
                            (b as f32 * 0.2) as u8,
                        )
                    }
                }
                _ => egui::Color32::GRAY,
            };
        }

        generated_colors
    }

    // Get generated colors with panic on uninitialized access
    fn get_colors(&self) -> &[egui::Color32; 12] {
        self.colors
            .as_ref()
            .expect("Colorix not initialized! Call generate_scale() first.")
    }

    // Color accessor methods
    pub fn app_background(&self) -> egui::Color32 {
        self.get_colors()[0]
    }

    pub fn subtle_background(&self) -> egui::Color32 {
        self.get_colors()[1]
    }

    pub fn ui_element_background(&self) -> egui::Color32 {
        self.get_colors()[2]
    }

    pub fn hovered_ui_element_background(&self) -> egui::Color32 {
        self.get_colors()[3]
    }

    pub fn active_ui_element_background(&self) -> egui::Color32 {
        self.get_colors()[4]
    }

    pub fn subtle_borders_and_separators(&self) -> egui::Color32 {
        self.get_colors()[5]
    }

    pub fn ui_element_border_and_focus_rings(&self) -> egui::Color32 {
        self.get_colors()[6]
    }

    pub fn hovered_ui_element_border(&self) -> egui::Color32 {
        self.get_colors()[7]
    }

    pub fn solid_backgrounds(&self) -> egui::Color32 {
        self.get_colors()[8]
    }

    pub fn hovered_solid_backgrounds(&self) -> egui::Color32 {
        self.get_colors()[9]
    }

    pub fn low_contrast_text(&self) -> egui::Color32 {
        self.get_colors()[10]
    }

    pub fn high_contrast_text(&self) -> egui::Color32 {
        self.get_colors()[11]
    }

    // Get color by index (0-11)
    pub fn color(&self, index: usize) -> egui::Color32 {
        self.get_colors()[index.min(11)]
    }

    pub fn set_egui_style(&self, style: &mut egui::style::Style) {
        let shadow = if self.dark_mode {
            egui::Color32::from_black_alpha(96)
        } else {
            egui::Color32::from_black_alpha(25)
        };
        let selection = egui::style::Selection {
            bg_fill: self.solid_backgrounds(),
            stroke: egui::Stroke::new(1.0, self.high_contrast_text()),
        };
        let text_cursor = egui::style::TextCursorStyle {
            stroke: egui::Stroke::new(2.0, self.low_contrast_text()),
            ..Default::default()
        };
        let widgets = egui::style::Widgets {
            noninteractive: egui::style::WidgetVisuals {
                weak_bg_fill: self.subtle_background(),
                bg_fill: self.subtle_background(),
                bg_stroke: egui::Stroke::new(1.0, self.subtle_borders_and_separators()), // separators, indentation lines
                fg_stroke: egui::Stroke::new(1.0, self.low_contrast_text()), // normal text color
                corner_radius: egui::CornerRadius::same(2),
                expansion: 0.0,
            },
            inactive: egui::style::WidgetVisuals {
                weak_bg_fill: self.ui_element_background(), // button background
                bg_fill: self.ui_element_background(),      // checkbox background
                bg_stroke: egui::Stroke::new(1.0, self.ui_element_background()),
                fg_stroke: egui::Stroke::new(1.0, self.low_contrast_text()), // button text
                corner_radius: egui::CornerRadius::same(2),
                expansion: 0.0,
            },
            hovered: egui::style::WidgetVisuals {
                weak_bg_fill: self.hovered_ui_element_background(),
                bg_fill: self.hovered_ui_element_background(),
                bg_stroke: egui::Stroke::new(1.0, self.hovered_ui_element_border()), // e.g. hover over window edge or button
                fg_stroke: egui::Stroke::new(1.5, self.high_contrast_text()),
                corner_radius: egui::CornerRadius::same(3),
                expansion: 1.0,
            },
            active: egui::style::WidgetVisuals {
                weak_bg_fill: self.active_ui_element_background(),
                bg_fill: self.active_ui_element_background(),
                bg_stroke: egui::Stroke::new(1.0, self.ui_element_border_and_focus_rings()),
                fg_stroke: egui::Stroke::new(2.0, self.high_contrast_text()),
                corner_radius: egui::CornerRadius::same(2),
                expansion: 1.0,
            },
            open: egui::style::WidgetVisuals {
                weak_bg_fill: self.active_ui_element_background(),
                bg_fill: self.active_ui_element_background(),
                bg_stroke: egui::Stroke::new(1.0, self.ui_element_border_and_focus_rings()),
                fg_stroke: egui::Stroke::new(1.0, self.high_contrast_text()),
                corner_radius: egui::CornerRadius::same(2),
                expansion: 0.0,
            },
        };
        style.visuals.selection = selection;
        style.visuals.widgets = widgets;
        style.visuals.text_cursor = text_cursor;
        style.visuals.extreme_bg_color = self.app_background(); // e.g. TextEdit background
        style.visuals.faint_bg_color = self.app_background(); // striped grid is originally from_additive_luminance(5)
        style.visuals.code_bg_color = self.ui_element_background();
        style.visuals.window_fill = self.subtle_background();
        style.visuals.window_stroke = egui::Stroke::new(1.0, self.subtle_borders_and_separators());
        style.visuals.panel_fill = self.subtle_background();
        style.visuals.hyperlink_color = self.hovered_solid_backgrounds();
        style.visuals.window_shadow.color = shadow;
    }

    // Convert app background to bevy color
    pub fn to_bevy_color(&self) -> bevy::color::Color {
        let color = self.app_background();
        bevy::color::Color::srgba_u8(color.r(), color.g(), color.b(), color.a())
    }

    // Apply background color to context
    pub fn draw_background(&self, ctx: &egui::Context, accent: bool) {
        let bg_color = if accent {
            self.solid_backgrounds()
        } else {
            self.app_background()
        };

        ctx.style_mut(|style| {
            style.visuals.panel_fill = bg_color;
            style.visuals.window_fill = bg_color;
            style.visuals.extreme_bg_color = bg_color;
        });
    }

    pub fn apply(&self, ctx: &egui::Context) {
        ctx.style_mut(|style| {
            self.set_egui_style(style);
            override_style(style);
        });
        init_style_map(self, &ctx.style());
    }

    pub fn save(self) {
        *COLORIX.lock() = self;
    }
}

static COLORIX: Mutex<Colorix> = Mutex::new(Colorix {
    raw_colors: [egui::Color32::GRAY; 12],
    dark_mode: true,
    colors: None,
});

pub fn colorix() -> MutexGuard<'static, Colorix> {
    COLORIX.lock()
}

// Convenience functions for common colors
pub fn app_background() -> egui::Color32 {
    COLORIX.lock().app_background()
}

pub fn subtle_background() -> egui::Color32 {
    COLORIX.lock().subtle_background()
}

pub fn ui_element_background() -> egui::Color32 {
    COLORIX.lock().ui_element_background()
}

pub fn subtle_borders_and_separators() -> egui::Color32 {
    COLORIX.lock().subtle_borders_and_separators()
}

pub fn ui_element_border_and_focus_rings() -> egui::Color32 {
    COLORIX.lock().ui_element_border_and_focus_rings()
}

pub fn hovered_ui_element_border() -> egui::Color32 {
    COLORIX.lock().hovered_ui_element_border()
}

pub fn solid_backgrounds() -> egui::Color32 {
    COLORIX.lock().solid_backgrounds()
}

pub fn low_contrast_text() -> egui::Color32 {
    COLORIX.lock().low_contrast_text()
}

pub fn high_contrast_text() -> egui::Color32 {
    COLORIX.lock().high_contrast_text()
}

fn override_style(style: &mut egui::Style) {
    style.visuals.widgets.active.corner_radius = ROUNDING;
    style.visuals.widgets.inactive.corner_radius = ROUNDING;
    style.visuals.widgets.hovered.corner_radius = ROUNDING;
    style.visuals.widgets.noninteractive.corner_radius = ROUNDING;
    style.visuals.widgets.open.corner_radius = ROUNDING;
}
