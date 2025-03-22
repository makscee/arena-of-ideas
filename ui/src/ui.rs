use egui::{CornerRadius, Margin, Shadow};

use super::*;

#[macro_export]
macro_rules! hex_color_noa {
    ($s:literal) => {{
        let array = color_hex::color_from_hex!($s);
        Color32::from_rgb(array[0], array[1], array[2])
    }};
}

pub const YELLOW: Color32 = hex_color_noa!("#D98F00");
pub const ORANGE: Color32 = hex_color_noa!("#DC6814");
pub const RED: Color32 = hex_color_noa!("#DC143C");
pub const DARK_RED: Color32 = hex_color_noa!("#9D0E2B");
pub const GREEN: Color32 = hex_color_noa!("#64DD17");
pub const PURPLE: Color32 = hex_color_noa!("#B50DA4");
pub const LIGHT_PURPLE: Color32 = hex_color_noa!("#95408D");
pub const CYAN: Color32 = hex_color_noa!("#00B8D4");

pub const MISSING_COLOR: Color32 = hex_color_noa!("#FF00FF");
pub const TRANSPARENT: Color32 = Color32::TRANSPARENT;
pub const CREDITS_SYM: char = '¤';

pub const SHADOW: Shadow = Shadow {
    offset: [8, 8],
    blur: 15,
    spread: 0,
    color: Color32::from_rgba_premultiplied(20, 20, 20, 35),
};
pub const MARGIN: Margin = Margin {
    left: 8,
    right: 8,
    top: 8,
    bottom: 8,
};

pub const ROUNDING: CornerRadius = CornerRadius {
    nw: 13,
    ne: 13,
    sw: 13,
    se: 13,
};

pub const UNIT_SIZE: f32 = 1.0;

pub fn dark_frame() -> Frame {
    Frame {
        inner_margin: Margin::same(5),
        outer_margin: Margin::same(5),
        corner_radius: ROUNDING,
        shadow: Shadow::NONE,
        fill: tokens_global().subtle_background(),
        stroke: Stroke::new(1.0, tokens_global().subtle_borders_and_separators()),
    }
}
