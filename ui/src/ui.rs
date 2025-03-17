use bevy::{app::Startup, prelude::Query};
use bevy_egui::{
    egui::epaint::text::{FontInsert, FontPriority, InsertFontFamily},
    EguiContext,
};
use egui::{
    style::{HandleShape, Spacing, WidgetVisuals, Widgets},
    CornerRadius, FontData, FontFamily, Margin, Shadow, Stroke,
};

use super::*;

#[macro_export]
macro_rules! hex_color_noa {
    ($s:literal) => {{
        let array = color_hex::color_from_hex!($s);
        Color32::from_rgb(array[0], array[1], array[2])
    }};
}

pub const EMPTINESS: Color32 = hex_color_noa!("#040404");
pub const BG_DARK: Color32 = hex_color_noa!("#161616");
pub const BG_LIGHT: Color32 = hex_color_noa!("#252525");
pub const BG_TRANSPARENT: Color32 = Color32::from_black_alpha(235);
pub const VISIBLE_DARK: Color32 = hex_color_noa!("#606060");
pub const VISIBLE_LIGHT: Color32 = hex_color_noa!("#B4B4B4");
pub const VISIBLE_BRIGHT: Color32 = hex_color_noa!("#FFFFFF");

pub const YELLOW: Color32 = hex_color_noa!("#D98F00");
pub const YELLOW_DARK: Color32 = hex_color_noa!("#996600");
pub const ORANGE: Color32 = hex_color_noa!("#DC6814");
pub const RED: Color32 = hex_color_noa!("#DC143C");
pub const DARK_RED: Color32 = hex_color_noa!("#9D0E2B");
pub const GREEN: Color32 = hex_color_noa!("#64DD17");
pub const PURPLE: Color32 = hex_color_noa!("#B50DA4");
pub const LIGHT_PURPLE: Color32 = hex_color_noa!("#95408D");
pub const CYAN: Color32 = hex_color_noa!("#00B8D4");

pub const EVENT_COLOR: Color32 = hex_color_noa!("#F7CA55");
pub const ACTION_COLOR: Color32 = hex_color_noa!("#DE1C1C");

pub const MISSING_COLOR: Color32 = hex_color_noa!("#FF00FF");

pub const TRANSPARENT: Color32 = Color32::TRANSPARENT;

pub const CREDITS_SYM: char = '¤';

pub const STROKE_LIGHT: Stroke = Stroke {
    width: 1.0,
    color: VISIBLE_LIGHT,
};
pub const STROKE_DARK: Stroke = Stroke {
    width: 1.0,
    color: VISIBLE_DARK,
};
pub const STROKE_BG_DARK: Stroke = Stroke {
    width: 1.0,
    color: BG_LIGHT,
};
pub const STROKE_YELLOW: Stroke = Stroke {
    width: 1.0,
    color: YELLOW,
};
pub const STROKE_YELLOW_DARK: Stroke = Stroke {
    width: 1.0,
    color: YELLOW_DARK,
};

pub const SHADOW: Shadow = Shadow {
    offset: [8, 8],
    blur: 15,
    spread: 0,
    color: Color32::from_rgba_premultiplied(20, 20, 20, 35),
};
pub const MARGIN: Margin = Margin {
    left: 4,
    right: 4,
    top: 4,
    bottom: 4,
};

pub const ROUNDING: CornerRadius = CornerRadius {
    nw: 13,
    ne: 13,
    sw: 13,
    se: 13,
};

pub const UNIT_SIZE: f32 = 1.0;

pub const DARK_FRAME: Frame = Frame {
    inner_margin: Margin::same(5),
    outer_margin: Margin::same(5),
    corner_radius: ROUNDING,
    shadow: Shadow::NONE,
    fill: TRANSPARENT,
    stroke: STROKE_BG_DARK,
};
