use parking_lot::{Mutex, MutexGuard};
use serde::{Deserialize, Serialize};

use super::*;
use crate::ui::see::cstr::init_style_map;
use bevy::prelude::Resource;

// Color space types for advanced color manipulation
#[derive(Debug, Clone, Copy)]
struct LinSrgb {
    r: f32,
    g: f32,
    b: f32,
}

#[derive(Debug, Clone, Copy)]
struct Okhsl {
    hue: f32,
    saturation: f32,
    lightness: f32,
}

impl LinSrgb {
    fn from_color32(color: egui::Color32) -> Self {
        let [r, g, b, _] = color.to_array();
        Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
        }
    }

    fn lighten(&self, factor: f32) -> Self {
        Self {
            r: (self.r + (1.0 - self.r) * factor).min(1.0),
            g: (self.g + (1.0 - self.g) * factor).min(1.0),
            b: (self.b + (1.0 - self.b) * factor).min(1.0),
        }
    }

    fn darken(&self, factor: f32) -> Self {
        Self {
            r: self.r * (1.0 - factor),
            g: self.g * (1.0 - factor),
            b: self.b * (1.0 - factor),
        }
    }
}

impl Okhsl {
    fn from_color(color: LinSrgb) -> Self {
        // Simplified conversion from RGB to OKHSL
        let max = color.r.max(color.g).max(color.b);
        let min = color.r.min(color.g).min(color.b);
        let delta = max - min;

        let lightness = (max + min) / 2.0;

        let saturation = if delta == 0.0 {
            0.0
        } else {
            delta / (1.0 - (2.0 * lightness - 1.0).abs())
        };

        let hue = if delta == 0.0 {
            0.0
        } else if max == color.r {
            60.0 * (((color.g - color.b) / delta) % 6.0)
        } else if max == color.g {
            60.0 * (((color.b - color.r) / delta) + 2.0)
        } else {
            60.0 * (((color.r - color.g) / delta) + 4.0)
        };

        Self {
            hue: hue.to_radians(),
            saturation: saturation.max(0.0).min(1.0),
            lightness: lightness.max(0.0).min(1.0),
        }
    }

    fn lighten(&self, factor: f32) -> Self {
        Self {
            hue: self.hue,
            saturation: self.saturation,
            lightness: (self.lightness + (1.0 - self.lightness) * factor).min(1.0),
        }
    }

    fn darken(&self, factor: f32) -> Self {
        Self {
            hue: self.hue,
            saturation: self.saturation,
            lightness: self.lightness * (1.0 - factor),
        }
    }

    fn as_degrees(&self) -> f32 {
        self.hue.to_degrees()
    }

    fn to_u8(&self) -> [u8; 3] {
        // Simplified conversion from OKHSL to RGB
        let c = (1.0 - (2.0 * self.lightness - 1.0).abs()) * self.saturation;
        let x = c * (1.0 - ((self.hue.to_degrees() / 60.0) % 2.0 - 1.0).abs());
        let m = self.lightness - c / 2.0;

        let (r, g, b) = if self.hue.to_degrees() < 60.0 {
            (c, x, 0.0)
        } else if self.hue.to_degrees() < 120.0 {
            (x, c, 0.0)
        } else if self.hue.to_degrees() < 180.0 {
            (0.0, c, x)
        } else if self.hue.to_degrees() < 240.0 {
            (0.0, x, c)
        } else if self.hue.to_degrees() < 300.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };

        [
            ((r + m) * 255.0) as u8,
            ((g + m) * 255.0) as u8,
            ((b + m) * 255.0) as u8,
        ]
    }
}

fn from_degrees(degrees: f32) -> f32 {
    degrees.to_radians()
}

fn estimate_lc(bg: egui::Color32, fg: egui::Color32) -> f32 {
    // Simplified contrast estimation
    let bg_lum = {
        let [r, g, b, _] = bg.to_array();
        0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32
    };
    let fg_lum = {
        let [r, g, b, _] = fg.to_array();
        0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32
    };

    (bg_lum - fg_lum) / 2.55 // Normalize to approximate APCA scale
}

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
        // Use the base color from slot 8 (solid_backgrounds) as the primary color
        let base_color = raw_colors[8];
        let srgb = LinSrgb::from_color32(base_color);

        let mut okhsl = [Okhsl::from_color(srgb); 12];
        let mut rgbs = [srgb; 12];
        let mut scale = [egui::Color32::GRAY; 12];

        let hsl = Okhsl::from_color(srgb);
        let hue = hsl.as_degrees();

        if dark_mode {
            // Dark mode scale generation
            rgbs[8] = srgb;
            okhsl[8] = hsl;

            let darken_values = [0.975, 0.96, 0.93, 0.89, 0.83, 0.75, 0.64, 0.39];
            let clamp_s = [0.3, 0.5, 0.8, 1., 1., 0.95, 0.7, 0.8];
            let clamp_s2 = [0.14, 0.16, 0.44, 0.62, 0.61, 0.56, 0.52, 0.51];
            let clamp_l = [0.08, 0.10, 0.15, 0.19, 0.23, 0.29, 0.36, 0.47];
            let lighten_values = [0.095, 0.45, 0.75];

            for i in 0..8 {
                rgbs[i] = srgb.darken(darken_values[i]);
                okhsl[i] = Okhsl::from_color(rgbs[i]);

                if (259.0..=323.).contains(&hue) {
                    okhsl[i] = okhsl[i].lighten((i + 1) as f32 * 0.011);
                }
                if (323.0..=350.).contains(&hue) && (i == 6 || i == 7) {
                    okhsl[i] = okhsl[i].lighten((i + 1) as f32 * 0.01);
                }

                okhsl[i].saturation *= 1.0 + ((1.0 - hsl.saturation) * 2.);

                if hsl.saturation > 0.36 {
                    okhsl[i].saturation = okhsl[i].saturation.clamp(
                        clamp_s2[i],
                        (hsl.saturation * clamp_s[i]).clamp(clamp_s2[i] + 0.01, 1.0),
                    );
                } else {
                    okhsl[i].saturation =
                        okhsl[i].saturation.clamp(0.0, hsl.saturation * clamp_s[i]);
                }

                okhsl[i].lightness = okhsl[i].lightness.clamp(
                    clamp_l[i],
                    (clamp_l[i] * (1.71 - hsl.saturation)).clamp(clamp_l[i] + 0.01, 1.0),
                );
            }

            for i in 9..12 {
                okhsl[i] = hsl.lighten(lighten_values[i - 9]);
                if (0.0..=90.).contains(&hue) || (300.0..=350.).contains(&hue) {
                    okhsl[i].hue = from_degrees(okhsl[i].as_degrees() + 2_f32 * (i - 8) as f32);
                }
                if (100.0..=280.).contains(&hue) {
                    okhsl[i].hue = from_degrees(okhsl[i].as_degrees() - 2_f32 * (i - 8) as f32);
                }
            }

            okhsl[10].lightness = okhsl[10].lightness.clamp(0.73, 1.0);
            okhsl[11].lightness = okhsl[11].lightness.clamp(0.88, 1.0);

            if (115.0..=220.).contains(&hue) {
                okhsl[11].saturation = okhsl[11].saturation.clamp(0.0, hsl.saturation * 0.75);
                okhsl[10].saturation = okhsl[10].saturation.clamp(0.0, hsl.saturation * 0.9);
            }

            let [x, y, z] = hsl.to_u8();
            let lc = estimate_lc(egui::Color32::WHITE, egui::Color32::from_rgb(x, y, z));
            if lc < -95.4 {
                okhsl[8] = hsl.lighten(0.3);
                okhsl[8].saturation = (hsl.saturation * 1.25).clamp(0., 1.);
                okhsl[9] = okhsl[9].lighten(0.25);
                okhsl[9].saturation = hsl.saturation;
            }
        } else {
            // Light mode scale generation
            rgbs[8] = srgb;
            okhsl[8] = hsl;

            let lighten_values = [0.965, 0.9, 0.82, 0.75, 0.63, 0.51, 0.39, 0.27];
            let clamp_v = [0.99, 0.98, 0.97, 0.95, 0.93, 0.90, 0.88, 0.85];
            let darken_values = [0.1, 0.2, 0.55];

            for (i, v) in lighten_values.iter().enumerate() {
                rgbs[i] = srgb.lighten(*v);
            }

            for i in 0..12 {
                if (0..9).contains(&i) {
                    okhsl[i] = Okhsl::from_color(rgbs[i]);
                    if i != 8 {
                        // adapt hue to compensate for temperature shift
                        if hue > 0. && hue < 90. {
                            okhsl[i].hue = from_degrees(okhsl[i].as_degrees() + 10_f32 - i as f32);
                        }
                        if hue > 200. && hue < 280. {
                            okhsl[i].hue = from_degrees(okhsl[i].as_degrees() - 10_f32 - i as f32);
                        }
                    }
                }
                if (9..12).contains(&i) {
                    okhsl[i] = Okhsl::from_color(srgb).darken(darken_values[i - 9]);
                }
                if i != 8 {
                    // enhance saturation for all values (except original) and diminish for certain hues (greenish)
                    let hue_u8 = hue as u8;
                    let sat_val = match hue_u8 {
                        159..=216 => ((hue_u8 - 159) as f32 / 58_f32) * 0.25,
                        100..=158 => ((158 - hue_u8) as f32 / 58_f32) * 0.25,
                        _ => 0.25,
                    };
                    let sat_clamp = match hue_u8 {
                        100..=158 => ((hue_u8 - 100) as f32 / 58_f32) * 0.12,
                        159..=217 => ((217 - hue_u8) as f32 / 58_f32) * 0.12,
                        _ => 0.0,
                    };
                    if hsl.saturation > 0.01 && hsl.lightness > 0.01 {
                        okhsl[i].saturation =
                            (hsl.saturation * hsl.lightness + sat_val).clamp(0.1, 1.0 - sat_clamp);
                    }
                    if i < 8 && hsl.lightness > 0.79 {
                        okhsl[i].lightness = okhsl[i].lightness.clamp(clamp_v[i] - 0.8, clamp_v[i]);
                    }
                }
            }

            okhsl[10].lightness = okhsl[10].lightness.clamp(0.43, 0.50);
            okhsl[11].lightness *= 0.9;

            let [x, y, z] = hsl.to_u8();
            let lc = estimate_lc(egui::Color32::WHITE, egui::Color32::from_rgb(x, y, z));
            if lc > -46. {
                okhsl[8].lightness = 0.68;
                okhsl[9].lightness = okhsl[8].lightness * 0.9;
                okhsl[9].saturation = okhsl[8].saturation * 0.9;
            } else {
                okhsl[9].saturation = okhsl[8].saturation;
            }
        }

        for i in 0..12 {
            let [r, g, b] = okhsl[i].to_u8();
            scale[i] = egui::Color32::from_rgb(r, g, b);
        }

        scale
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
                corner_radius: ROUNDING,
                expansion: 0.0,
            },
            inactive: egui::style::WidgetVisuals {
                weak_bg_fill: self.ui_element_background(), // button background
                bg_fill: self.ui_element_background(),      // checkbox background
                bg_stroke: egui::Stroke::new(1.0, self.ui_element_background()),
                fg_stroke: egui::Stroke::new(1.0, self.low_contrast_text()), // button text
                corner_radius: ROUNDING,
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
                corner_radius: ROUNDING,
                expansion: 1.0,
            },
            open: egui::style::WidgetVisuals {
                weak_bg_fill: self.active_ui_element_background(),
                bg_fill: self.active_ui_element_background(),
                bg_stroke: egui::Stroke::new(1.0, self.ui_element_border_and_focus_rings()),
                fg_stroke: egui::Stroke::new(1.0, self.high_contrast_text()),
                corner_radius: ROUNDING,
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

    pub fn apply(&self, ctx: &egui::Context) {
        ctx.style_mut(|style| {
            self.set_egui_style(style);
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
