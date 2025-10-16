use parking_lot::{Mutex, MutexGuard};
use serde::{Deserialize, Serialize};

use super::*;
use crate::ui::core::cstr::init_style_map;
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
    let bg_lum = {
        let [r, g, b, _] = bg.to_array();
        0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32
    };
    let fg_lum = {
        let [r, g, b, _] = fg.to_array();
        0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32
    };

    (bg_lum - fg_lum) / 2.55
}

// Radix preset colors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, AsRefStr, EnumIter)]
pub enum RadixColor {
    Tomato,
    Red,
    Ruby,
    Crimson,
    Pink,
    Plum,
    Purple,
    Violet,
    Iris,
    Indigo,
    Blue,
    Cyan,
    Teal,
    Jade,
    Green,
    Grass,
    Bronze,
    Gold,
    Brown,
    Orange,
    Amber,
    Yellow,
    Lime,
    Mint,
    Sky,
    Gray,
    Slate,
    Custom(egui::Color32),
}

impl ToCstr for RadixColor {
    fn cstr(&self) -> Cstr {
        format!("[{} {}]", self.to_color32().to_hex(), self.as_ref())
    }
}

impl RadixColor {
    pub fn to_color32(&self) -> egui::Color32 {
        match self {
            Self::Tomato => egui::Color32::from_rgb(229, 77, 46),
            Self::Red => egui::Color32::from_rgb(229, 72, 77),
            Self::Ruby => egui::Color32::from_rgb(229, 70, 102),
            Self::Crimson => egui::Color32::from_rgb(233, 61, 130),
            Self::Pink => egui::Color32::from_rgb(214, 64, 159),
            Self::Plum => egui::Color32::from_rgb(171, 74, 186),
            Self::Purple => egui::Color32::from_rgb(142, 78, 198),
            Self::Violet => egui::Color32::from_rgb(117, 91, 223),
            Self::Iris => egui::Color32::from_rgb(91, 91, 214),
            Self::Indigo => egui::Color32::from_rgb(62, 99, 221),
            Self::Blue => egui::Color32::from_rgb(0, 112, 243),
            Self::Cyan => egui::Color32::from_rgb(0, 157, 196),
            Self::Teal => egui::Color32::from_rgb(18, 165, 148),
            Self::Jade => egui::Color32::from_rgb(29, 167, 139),
            Self::Green => egui::Color32::from_rgb(48, 164, 108),
            Self::Grass => egui::Color32::from_rgb(70, 167, 88),
            Self::Bronze => egui::Color32::from_rgb(161, 128, 114),
            Self::Gold => egui::Color32::from_rgb(151, 117, 68),
            Self::Brown => egui::Color32::from_rgb(173, 127, 88),
            Self::Orange => egui::Color32::from_rgb(247, 107, 21),
            Self::Amber => egui::Color32::from_rgb(255, 190, 10),
            Self::Yellow => egui::Color32::from_rgb(252, 211, 78),
            Self::Lime => egui::Color32::from_rgb(189, 222, 61),
            Self::Mint => egui::Color32::from_rgb(134, 225, 187),
            Self::Sky => egui::Color32::from_rgb(117, 199, 240),
            Self::Gray => egui::Color32::from_rgb(141, 141, 141),
            Self::Slate => egui::Color32::from_rgb(136, 144, 162),
            Self::Custom(color) => *color,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, AsRefStr)]
pub enum Semantic {
    Accent,
    Success,
    Error,
    Warning,
    Background,
}

// A 12-step color palette
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorPalette {
    colors: [egui::Color32; 12],
}

impl ColorPalette {
    fn new(base_color: egui::Color32, dark_mode: bool) -> Self {
        let colors = Self::generate_color_scale(base_color, dark_mode);
        Self { colors }
    }

    fn generate_color_scale(base_color: egui::Color32, dark_mode: bool) -> [egui::Color32; 12] {
        let srgb = LinSrgb::from_color32(base_color);
        let mut okhsl = [Okhsl::from_color(srgb); 12];
        let mut rgbs = [srgb; 12];
        let mut scale = [egui::Color32::GRAY; 12];

        let hsl = Okhsl::from_color(srgb);
        let hue = hsl.as_degrees();

        if dark_mode {
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

    // Named accessors for color steps
    pub fn bg_0(&self) -> egui::Color32 {
        self.colors[0]
    }

    pub fn bg_1(&self) -> egui::Color32 {
        self.colors[1]
    }

    pub fn interactive_0(&self) -> egui::Color32 {
        self.colors[2]
    }

    pub fn interactive_1(&self) -> egui::Color32 {
        self.colors[3]
    }

    pub fn interactive_2(&self) -> egui::Color32 {
        self.colors[4]
    }

    pub fn border_0(&self) -> egui::Color32 {
        self.colors[5]
    }

    pub fn border_1(&self) -> egui::Color32 {
        self.colors[6]
    }

    pub fn border_2(&self) -> egui::Color32 {
        self.colors[7]
    }

    pub fn solid_0(&self) -> egui::Color32 {
        self.colors[8]
    }

    pub fn solid_1(&self) -> egui::Color32 {
        self.colors[9]
    }

    pub fn text_0(&self) -> egui::Color32 {
        self.colors[10]
    }

    pub fn text_1(&self) -> egui::Color32 {
        self.colors[11]
    }

    // Indexed accessor
    pub fn step(&self, index: usize) -> egui::Color32 {
        self.colors[index.min(11)]
    }
}

// Color theme definition
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct ColorTheme {
    pub accent: RadixColor,
    pub success: RadixColor,
    pub error: RadixColor,
    pub warning: RadixColor,
    pub background: RadixColor,
}

impl Default for ColorTheme {
    fn default() -> Self {
        Self {
            accent: RadixColor::Blue,
            error: RadixColor::Red,
            success: RadixColor::Green,
            warning: RadixColor::Amber,
            background: RadixColor::Slate,
        }
    }
}

impl ColorTheme {
    pub fn get_color(&self, semantic: Semantic) -> RadixColor {
        match semantic {
            Semantic::Accent => self.accent,
            Semantic::Error => self.error,
            Semantic::Success => self.success,
            Semantic::Warning => self.warning,
            Semantic::Background => self.background,
        }
    }
    pub fn set_color(&mut self, semantic: Semantic, color: RadixColor) {
        match semantic {
            Semantic::Accent => self.accent = color,
            Semantic::Error => self.error = color,
            Semantic::Success => self.success = color,
            Semantic::Warning => self.warning = color,
            Semantic::Background => self.background = color,
        }
    }
}

// Main Colorix struct
#[derive(Resource, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Colorix {
    theme: ColorTheme,
    dark_mode: bool,

    #[serde(skip)]
    palettes: Option<SemanticPalettes>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SemanticPalettes {
    accent: ColorPalette,
    error: ColorPalette,
    success: ColorPalette,
    warning: ColorPalette,
    background: ColorPalette,
}

impl Default for Colorix {
    fn default() -> Self {
        let mut colorix = Self {
            theme: ColorTheme::default(),
            dark_mode: true,
            palettes: None,
        };
        colorix.generate_palettes();
        colorix
    }
}

impl Colorix {
    pub fn new(base_color: egui::Color32, dark_mode: bool) -> Self {
        let theme = ColorTheme {
            accent: RadixColor::Custom(base_color),
            error: RadixColor::Red,
            success: RadixColor::Green,
            warning: RadixColor::Amber,
            background: RadixColor::Custom(base_color),
        };
        Self::with_theme(theme, dark_mode)
    }

    pub fn with_theme(theme: ColorTheme, dark_mode: bool) -> Self {
        let mut colorix = Self {
            theme,
            dark_mode,
            palettes: None,
        };
        colorix.generate_palettes();
        colorix
    }

    pub fn set_theme(&mut self, theme: ColorTheme) -> &mut Self {
        self.theme = theme;
        self.generate_palettes();
        self
    }

    pub fn set_dark_mode(&mut self, dark_mode: bool) -> &mut Self {
        self.dark_mode = dark_mode;
        self.generate_palettes();
        self
    }

    pub fn dark_mode(&self) -> bool {
        self.dark_mode
    }

    pub fn theme(&self) -> ColorTheme {
        self.theme
    }

    // Alias for generate_palettes for backward compatibility
    pub fn generate_scale(&mut self) -> &mut Self {
        self.generate_palettes()
    }

    pub fn generate_palettes(&mut self) -> &mut Self {
        let palettes = SemanticPalettes {
            accent: ColorPalette::new(self.theme.accent.to_color32(), self.dark_mode),
            error: ColorPalette::new(self.theme.error.to_color32(), self.dark_mode),
            success: ColorPalette::new(self.theme.success.to_color32(), self.dark_mode),
            warning: ColorPalette::new(self.theme.warning.to_color32(), self.dark_mode),
            background: ColorPalette::new(self.theme.background.to_color32(), self.dark_mode),
        };
        self.palettes = Some(palettes);
        self
    }

    fn get_palettes(&self) -> &SemanticPalettes {
        self.palettes
            .as_ref()
            .expect("Colorix not initialized! Call generate_palettes() first.")
    }

    fn color_dropdown(ui: &mut egui::Ui, color: &mut RadixColor) -> bool {
        let mut changed = Selector::ui_enum(color, ui).0.is_some();
        if let RadixColor::Custom(c) = color {
            if c.edit(ui).changed() {
                changed = true;
            }
        }

        changed
    }

    fn show_palette_squares(ui: &mut egui::Ui, palette: &ColorPalette) {
        ui.horizontal(|ui| {
            for i in 0..12 {
                let color = palette.step(i);
                let size = egui::vec2(12.0, 12.0);
                let (rect, response) = ui.allocate_exact_size(size, Sense::HOVER);
                ui.painter().rect_filled(rect, 1.0, color);

                if response.hovered() {
                    let tooltip_text = match i {
                        0 => "bg_0",
                        1 => "bg_1",
                        2 => "interactive_0",
                        3 => "interactive_1",
                        4 => "interactive_2",
                        5 => "border_0",
                        6 => "border_1",
                        7 => "border_2",
                        8 => "solid_0",
                        9 => "solid_1",
                        10 => "text_0",
                        11 => "text_1",
                        _ => "unknown",
                    };
                    response.on_hover_text(tooltip_text);
                }
            }
        });
    }

    pub fn show_semantic_editor(&mut self, semantic: Semantic, ui: &mut Ui) -> bool {
        let mut changed = false;
        ui.group(|ui| {
            ui.label(semantic.as_ref());
            let mut color = self.theme().get_color(semantic);
            ui.horizontal(|ui| {
                if Self::color_dropdown(ui, &mut color) {
                    let mut theme = self.theme();
                    theme.set_color(semantic, color);
                    self.set_theme(theme);
                    changed = true;
                }
                Self::show_palette_squares(ui, self.get_palette(semantic));
            });
        });
        changed
    }

    pub fn get_palette(&self, semantic: Semantic) -> &ColorPalette {
        let palettes = self.get_palettes();
        match semantic {
            Semantic::Accent => &palettes.accent,
            Semantic::Success => &palettes.success,
            Semantic::Error => &palettes.error,
            Semantic::Warning => &palettes.warning,
            Semantic::Background => &palettes.background,
        }
    }

    pub fn accent(&self) -> &ColorPalette {
        &self.get_palettes().accent
    }

    pub fn error(&self) -> &ColorPalette {
        &self.get_palettes().error
    }

    pub fn success(&self) -> &ColorPalette {
        &self.get_palettes().success
    }

    pub fn warning(&self) -> &ColorPalette {
        &self.get_palettes().warning
    }

    pub fn background(&self) -> &ColorPalette {
        &self.get_palettes().background
    }

    // Legacy compatibility methods
    pub fn app_background(&self) -> egui::Color32 {
        self.background().bg_0()
    }

    pub fn subtle_background(&self) -> egui::Color32 {
        self.background().bg_1()
    }

    pub fn ui_element_background(&self) -> egui::Color32 {
        self.background().interactive_0()
    }

    pub fn hovered_ui_element_background(&self) -> egui::Color32 {
        self.background().interactive_1()
    }

    pub fn active_ui_element_background(&self) -> egui::Color32 {
        self.background().interactive_2()
    }

    pub fn subtle_borders_and_separators(&self) -> egui::Color32 {
        self.background().border_0()
    }

    pub fn ui_element_border_and_focus_rings(&self) -> egui::Color32 {
        self.background().border_1()
    }

    pub fn hovered_ui_element_border(&self) -> egui::Color32 {
        self.background().border_2()
    }

    pub fn solid_backgrounds(&self) -> egui::Color32 {
        self.accent().solid_0()
    }

    pub fn hovered_solid_backgrounds(&self) -> egui::Color32 {
        self.accent().solid_1()
    }

    pub fn low_contrast_text(&self) -> egui::Color32 {
        self.background().text_0()
    }

    pub fn high_contrast_text(&self) -> egui::Color32 {
        self.background().text_1()
    }

    // Get color by index (0-11) for backward compatibility
    pub fn color(&self, index: usize) -> egui::Color32 {
        self.background().step(index)
    }

    // Raw colors getter for backward compatibility
    pub fn raw_colors(&self) -> [egui::Color32; 12] {
        let bg = self.background();
        [
            bg.bg_0(),
            bg.bg_1(),
            bg.interactive_0(),
            bg.interactive_1(),
            bg.interactive_2(),
            bg.border_0(),
            bg.border_1(),
            bg.border_2(),
            bg.solid_0(),
            bg.solid_1(),
            bg.text_0(),
            bg.text_1(),
        ]
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
                bg_stroke: egui::Stroke::new(1.0, self.subtle_borders_and_separators()),
                fg_stroke: egui::Stroke::new(1.0, self.low_contrast_text()),
                corner_radius: ROUNDING,
                expansion: 0.0,
            },
            inactive: egui::style::WidgetVisuals {
                weak_bg_fill: self.ui_element_background(),
                bg_fill: self.ui_element_background(),
                bg_stroke: egui::Stroke::new(1.0, self.ui_element_background()),
                fg_stroke: egui::Stroke::new(1.0, self.low_contrast_text()),
                corner_radius: ROUNDING,
                expansion: 0.0,
            },
            hovered: egui::style::WidgetVisuals {
                weak_bg_fill: self.hovered_ui_element_background(),
                bg_fill: self.hovered_ui_element_background(),
                bg_stroke: egui::Stroke::new(1.0, self.hovered_ui_element_border()),
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
        style.visuals.extreme_bg_color = self.app_background();
        style.visuals.faint_bg_color = self.app_background();
        style.visuals.code_bg_color = self.ui_element_background();
        style.visuals.window_fill = self.subtle_background();
        style.visuals.window_stroke = egui::Stroke::new(1.0, self.subtle_borders_and_separators());
        style.visuals.panel_fill = self.subtle_background();
        style.visuals.hyperlink_color = self.hovered_solid_backgrounds();
        style.visuals.window_shadow.color = shadow;
    }

    pub fn set_egui_style_for_semantic(&self, style: &mut egui::style::Style, semantic: Semantic) {
        let palette = match semantic {
            Semantic::Accent => &self.get_palettes().accent,
            Semantic::Error => &self.get_palettes().error,
            Semantic::Success => &self.get_palettes().success,
            Semantic::Warning => &self.get_palettes().warning,
            Semantic::Background => &self.get_palettes().background,
        };

        let shadow = if self.dark_mode {
            egui::Color32::from_black_alpha(96)
        } else {
            egui::Color32::from_black_alpha(25)
        };
        let selection = egui::style::Selection {
            bg_fill: palette.solid_0(),
            stroke: egui::Stroke::new(1.0, palette.text_1()),
        };
        let text_cursor = egui::style::TextCursorStyle {
            stroke: egui::Stroke::new(2.0, palette.text_0()),
            ..Default::default()
        };
        let widgets = egui::style::Widgets {
            noninteractive: egui::style::WidgetVisuals {
                weak_bg_fill: palette.bg_1(),
                bg_fill: palette.bg_1(),
                bg_stroke: egui::Stroke::new(1.0, palette.border_0()),
                fg_stroke: egui::Stroke::new(1.0, palette.text_0()),
                corner_radius: ROUNDING,
                expansion: 0.0,
            },
            inactive: egui::style::WidgetVisuals {
                weak_bg_fill: palette.interactive_0(),
                bg_fill: palette.interactive_0(),
                bg_stroke: egui::Stroke::new(1.0, palette.border_0()),
                fg_stroke: egui::Stroke::new(1.0, palette.text_0()),
                corner_radius: ROUNDING,
                expansion: 0.0,
            },
            hovered: egui::style::WidgetVisuals {
                weak_bg_fill: palette.interactive_1(),
                bg_fill: palette.interactive_1(),
                bg_stroke: egui::Stroke::new(1.0, palette.border_2()),
                fg_stroke: egui::Stroke::new(1.5, palette.text_1()),
                corner_radius: egui::CornerRadius::same(3),
                expansion: 1.0,
            },
            active: egui::style::WidgetVisuals {
                weak_bg_fill: palette.interactive_2(),
                bg_fill: palette.interactive_2(),
                bg_stroke: egui::Stroke::new(1.0, palette.border_1()),
                fg_stroke: egui::Stroke::new(2.0, palette.text_1()),
                corner_radius: ROUNDING,
                expansion: 1.0,
            },
            open: egui::style::WidgetVisuals {
                weak_bg_fill: palette.interactive_2(),
                bg_fill: palette.interactive_2(),
                bg_stroke: egui::Stroke::new(1.0, palette.border_1()),
                fg_stroke: egui::Stroke::new(1.0, palette.text_1()),
                corner_radius: ROUNDING,
                expansion: 0.0,
            },
        };
        style.visuals.selection = selection;
        style.visuals.widgets = widgets;
        style.visuals.text_cursor = text_cursor;
        style.visuals.extreme_bg_color = palette.bg_0();
        style.visuals.faint_bg_color = palette.bg_0();
        style.visuals.code_bg_color = palette.interactive_0();
        style.visuals.window_fill = palette.bg_1();
        style.visuals.window_stroke = egui::Stroke::new(1.0, palette.border_0());
        style.visuals.panel_fill = palette.bg_1();
        style.visuals.hyperlink_color = palette.solid_1();
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

// Extension trait for Ui to apply semantic coloring
pub trait UiColorixExt {
    fn colorix_semantic<R>(&mut self, semantic: Semantic, f: impl FnOnce(&mut egui::Ui) -> R) -> R;
}

impl UiColorixExt for egui::Ui {
    fn colorix_semantic<R>(&mut self, semantic: Semantic, f: impl FnOnce(&mut egui::Ui) -> R) -> R {
        let original_style = (*self.ctx().style()).clone();
        self.ctx().style_mut(|style| {
            colorix().set_egui_style_for_semantic(style, semantic);
        });
        let result = f(self);
        self.ctx().set_style(original_style);
        result
    }
}

static COLORIX: Mutex<Colorix> = Mutex::new(Colorix {
    theme: ColorTheme {
        accent: RadixColor::Blue,
        error: RadixColor::Red,
        success: RadixColor::Green,
        warning: RadixColor::Amber,
        background: RadixColor::Slate,
    },
    dark_mode: true,
    palettes: None,
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
