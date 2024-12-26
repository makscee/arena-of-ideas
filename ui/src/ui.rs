use bevy::{app::Startup, prelude::Query};
use bevy_egui::EguiContext;
use egui::{
    style::{HandleShape, Spacing, WidgetVisuals, Widgets},
    FontData, FontDefinitions, FontFamily, LayerId, Margin, Rounding, Shadow, Stroke,
};

use super::*;

#[macro_export]
macro_rules! hex_color_noa {
    ($s:literal) => {{
        let array = color_hex::color_from_hex!($s);
        Color32::from_rgb(array[0], array[1], array[2])
    }};
}

pub const EMPTINESS: Color32 = hex_color_noa!("#080808");
pub const BG_DARK: Color32 = hex_color_noa!("#191919");
pub const BG_LIGHT: Color32 = hex_color_noa!("#252525");
pub const BG_TRANSPARENT: Color32 = Color32::from_black_alpha(235);
pub const VISIBLE_DARK: Color32 = hex_color_noa!("#606060");
pub const VISIBLE_LIGHT: Color32 = hex_color_noa!("#B4B4B4");
pub const VISIBLE_BRIGHT: Color32 = hex_color_noa!("#FFFFFF");

pub const YELLOW: Color32 = hex_color_noa!("#D98F00");
pub const YELLOW_DARK: Color32 = hex_color_noa!("#493501");
pub const RED: Color32 = hex_color_noa!("#DC143C");
pub const DARK_RED: Color32 = hex_color_noa!("#9D0E2B");
pub const GREEN: Color32 = hex_color_noa!("#64DD17");
pub const PURPLE: Color32 = hex_color_noa!("#B50DA4");
pub const LIGHT_PURPLE: Color32 = hex_color_noa!("#95408D");
pub const CYAN: Color32 = hex_color_noa!("#00B8D4");

pub const EVENT_COLOR: Color32 = hex_color_noa!("#F7CA55");
pub const TARGET_COLOR: Color32 = hex_color_noa!("#DC6814");
pub const EFFECT_COLOR: Color32 = hex_color_noa!("#DE1C1C");

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
pub const STROKE_YELLOW: Stroke = Stroke {
    width: 1.0,
    color: YELLOW,
};

pub const SHADOW: Shadow = Shadow {
    offset: egui::vec2(8.0, 8.0),
    blur: 15.0,
    spread: 0.0,
    color: Color32::from_rgba_premultiplied(20, 20, 20, 35),
};
pub const MARGIN: Margin = Margin {
    left: 4.0,
    right: 4.0,
    top: 4.0,
    bottom: 4.0,
};

pub const ROUNDING: Rounding = Rounding {
    nw: 13.0,
    ne: 13.0,
    sw: 13.0,
    se: 13.0,
};

pub const UNIT_SIZE: f32 = 0.8;

pub fn empty_response(ctx: egui::Context) -> Response {
    Response {
        ctx,
        layer_id: LayerId::background(),
        id: Id::new(0),
        rect: Rect::ZERO,
        interact_rect: Rect::ZERO,
        sense: Sense::hover(),
        enabled: false,
        contains_pointer: false,
        hovered: false,
        highlighted: false,
        clicked: false,
        fake_primary_click: false,
        long_touched: false,
        drag_started: false,
        dragged: false,
        drag_stopped: false,
        is_pointer_button_down_on: false,
        interact_pointer_pos: None,
        changed: false,
    }
}

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
        FontData::from_static(include_bytes!(
            "../../assets/fonts/SometypeMono-Regular.ttf"
        )),
    );
    fonts.font_data.insert(
        "medium".to_owned(),
        FontData::from_static(include_bytes!("../../assets/fonts/SometypeMono-Medium.ttf")),
    );
    fonts.font_data.insert(
        "bold".to_owned(),
        FontData::from_static(include_bytes!("../../assets/fonts/SometypeMono-Bold.ttf")),
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
            (TextStyle::Button, FontId::new(16.0, FontFamily::Monospace)),
            (TextStyle::Small, FontId::new(10.0, FontFamily::Monospace)),
        ]
        .into();
        style.animation_time = 0.1;
        style.spacing = Spacing {
            item_spacing: egui::vec2(8.0, 6.0),
            button_padding: egui::vec2(3.0, 3.0),
            combo_width: 10.0,
            ..Default::default()
        };
        style.wrap_mode = Some(egui::TextWrapMode::Extend);
        style.spacing.window_margin = Margin::same(13.0);
        style.spacing.slider_rail_height = 2.0;
        style.spacing.button_padding = egui::vec2(8.0, 2.0);
        style.visuals.striped = false;
        style.visuals.slider_trailing_fill = true;
        style.visuals.faint_bg_color = BG_LIGHT;
        style.visuals.handle_shape = HandleShape::Rect { aspect_ratio: 0.1 };
        style.visuals.selection.bg_fill = YELLOW_DARK;
        style.visuals.resize_corner_size = 0.0;
        style.visuals.window_stroke = Stroke {
            width: 1.0,
            color: VISIBLE_DARK,
        };
        style.visuals.window_fill = BG_DARK;
        style.visuals.extreme_bg_color = EMPTINESS;
        style.visuals.widgets = Widgets {
            noninteractive: WidgetVisuals {
                weak_bg_fill: Color32::TRANSPARENT,
                bg_fill: Color32::from_gray(27),
                bg_stroke: Stroke::new(1.0, VISIBLE_DARK), // separators, indentation lines
                fg_stroke: Stroke::new(1.0, VISIBLE_DARK), // normal text color
                rounding: Rounding::same(13.0),
                expansion: 0.0,
            },
            inactive: WidgetVisuals {
                weak_bg_fill: Color32::TRANSPARENT,
                bg_fill: BG_LIGHT, // checkbox background
                bg_stroke: Stroke::new(1.0, BG_LIGHT),
                fg_stroke: Stroke::new(1.0, VISIBLE_LIGHT), // button text
                rounding: Rounding::same(13.0),
                expansion: 0.0,
            },
            hovered: WidgetVisuals {
                weak_bg_fill: Color32::TRANSPARENT,
                bg_fill: Color32::from_gray(70),
                bg_stroke: Stroke::new(1.0, VISIBLE_LIGHT), // e.g. hover over window edge or button
                fg_stroke: Stroke::new(1.5, VISIBLE_BRIGHT),
                rounding: Rounding::same(13.0),
                expansion: 0.0,
            },
            active: WidgetVisuals {
                weak_bg_fill: Color32::TRANSPARENT,
                bg_fill: Color32::from_gray(55),
                bg_stroke: Stroke::new(1.0, YELLOW),
                fg_stroke: Stroke::new(2.0, YELLOW),
                rounding: Rounding::same(13.0),
                expansion: 0.0,
            },
            open: WidgetVisuals {
                weak_bg_fill: Color32::from_gray(45),
                bg_fill: Color32::from_gray(27),
                bg_stroke: Stroke::new(1.0, Color32::from_gray(60)),
                fg_stroke: Stroke::new(1.0, Color32::from_gray(210)),
                rounding: Rounding::same(13.0),
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
