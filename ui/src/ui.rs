use std::sync::Arc;

use bevy::{app::Startup, prelude::Query};
use bevy_egui::EguiContext;
use egui::{
    style::{HandleShape, Spacing, WidgetVisuals, Widgets},
    CornerRadius, FontData, FontDefinitions, FontFamily, LayerId, Margin, Shadow, Stroke,
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

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_ui);
    }
}

fn setup_ui(mut ctx: Query<&mut EguiContext>) {
    let ctx = ctx.single_mut().into_inner().get_mut();
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        "regular".to_owned(),
        Arc::new(FontData::from_static(include_bytes!(
            "../../assets/fonts/SometypeMono-Regular.ttf"
        ))),
    );
    fonts.font_data.insert(
        "medium".to_owned(),
        Arc::new(FontData::from_static(include_bytes!(
            "../../assets/fonts/SometypeMono-Medium.ttf"
        ))),
    );
    fonts.font_data.insert(
        "bold".to_owned(),
        Arc::new(FontData::from_static(include_bytes!(
            "../../assets/fonts/SometypeMono-Bold.ttf"
        ))),
    );
    fonts
        .families
        .get_mut(&FontFamily::Monospace)
        .unwrap()
        .insert(0, "regular".to_owned());
    fonts
        .families
        .get_mut(&FontFamily::Proportional)
        .unwrap()
        .insert(0, "regular".to_owned());
    fonts
        .families
        .insert(FontFamily::Name("medium".into()), vec!["medium".to_owned()]);
    fonts
        .families
        .insert(FontFamily::Name("bold".into()), vec!["bold".to_owned()]);
    ctx.set_fonts(fonts);
    ctx.style_mut(|style| {
        style.text_styles = [
            (
                TextStyle::Heading,
                FontId::new(26.0, FontFamily::Name("medium".into())),
            ),
            (
                TextStyle::Name("Heading2".into()),
                FontId::new(22.0, FontFamily::Monospace),
            ),
            (
                TextStyle::Name("Bold".into()),
                FontId::new(14.0, FontFamily::Name("bold".into())),
            ),
            (
                TextStyle::Name("Context".into()),
                FontId::new(20.0, FontFamily::Monospace),
            ),
            (TextStyle::Body, FontId::new(14.0, FontFamily::Monospace)),
            (
                TextStyle::Monospace,
                FontId::new(14.0, FontFamily::Monospace),
            ),
            (TextStyle::Button, FontId::new(14.0, FontFamily::Monospace)),
            (TextStyle::Small, FontId::new(10.0, FontFamily::Monospace)),
        ]
        .into();
        style.animation_time = 0.1;
        style.spacing = Spacing {
            item_spacing: egui::vec2(8.0, 6.0),
            button_padding: egui::vec2(3.0, 3.0),
            combo_width: 10.0,
            interact_size: egui::vec2(20.0, 20.0),
            ..Default::default()
        };
        style.wrap_mode = Some(egui::TextWrapMode::Extend);
        style.spacing.window_margin = Margin::same(0);
        style.spacing.slider_rail_height = 2.0;
        style.spacing.button_padding = egui::vec2(8.0, 2.0);
        style.visuals.panel_fill = EMPTINESS;
        style.visuals.striped = false;
        style.visuals.slider_trailing_fill = true;
        style.visuals.faint_bg_color = BG_LIGHT;
        style.visuals.handle_shape = HandleShape::Rect { aspect_ratio: 0.1 };
        style.visuals.selection.bg_fill = YELLOW_DARK;
        style.visuals.resize_corner_size = 0.0;
        style.visuals.window_stroke = Stroke::NONE;
        style.visuals.window_fill = TRANSPARENT;
        style.visuals.extreme_bg_color = TRANSPARENT;
        style.visuals.widgets = Widgets {
            noninteractive: WidgetVisuals {
                weak_bg_fill: Color32::TRANSPARENT,
                bg_fill: Color32::from_gray(27),
                bg_stroke: STROKE_DARK, // separators, indentation lines
                fg_stroke: Stroke::new(1.0, VISIBLE_DARK), // normal text color
                corner_radius: CornerRadius::same(13),
                expansion: 0.0,
            },
            inactive: WidgetVisuals {
                weak_bg_fill: Color32::TRANSPARENT,
                bg_fill: BG_LIGHT, // checkbox background
                bg_stroke: STROKE_DARK,
                fg_stroke: Stroke::new(1.0, VISIBLE_LIGHT), // button text
                corner_radius: CornerRadius::same(13),
                expansion: 0.0,
            },
            hovered: WidgetVisuals {
                weak_bg_fill: Color32::TRANSPARENT,
                bg_fill: Color32::from_gray(70),
                bg_stroke: STROKE_LIGHT, // e.g. hover over window edge or button
                fg_stroke: Stroke::new(1.5, VISIBLE_BRIGHT),
                corner_radius: CornerRadius::same(13),
                expansion: 0.0,
            },
            active: WidgetVisuals {
                weak_bg_fill: Color32::TRANSPARENT,
                bg_fill: Color32::from_gray(55),
                bg_stroke: STROKE_YELLOW,
                fg_stroke: Stroke::new(2.0, YELLOW),
                corner_radius: CornerRadius::same(13),
                expansion: 0.0,
            },
            open: WidgetVisuals {
                weak_bg_fill: Color32::from_gray(45),
                bg_fill: Color32::from_gray(27),
                bg_stroke: STROKE_DARK,
                fg_stroke: Stroke::new(1.0, Color32::from_gray(210)),
                corner_radius: CornerRadius::same(13),
                expansion: 0.0,
            },
        };
    });
}

#[derive(Default, Debug, PartialEq, Eq)]
pub enum Side {
    #[default]
    Right,
    Left,
    Top,
    Bottom,
}

impl Side {
    pub fn is_x(&self) -> bool {
        match self {
            Side::Right | Side::Left => true,
            Side::Top | Side::Bottom => false,
        }
    }
    pub fn is_y(&self) -> bool {
        match self {
            Side::Top | Side::Bottom => true,
            Side::Right | Side::Left => false,
        }
    }
}
