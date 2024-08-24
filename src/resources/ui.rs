use egui::{FontId, TextStyle};

use super::*;

#[macro_export]
macro_rules! hex_color_noa {
    ($s:literal) => {{
        let array = color_hex::color_from_hex!($s);
        $crate::Color32::from_rgb(array[0], array[1], array[2])
    }};
}

pub const EMPTINESS: Color32 = hex_color_noa!("#080808");
pub const BG_DARK: Color32 = hex_color_noa!("#191919");
pub const BG_LIGHT: Color32 = hex_color_noa!("#252525");
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
pub const BEVY_MISSING_COLOR: LinearRgba = LinearRgba::new(1.0, 0.0, 1.0, 1.0);

pub const TRANSPARENT: Color32 = Color32::TRANSPARENT;

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
    offset: egui::vec2(10.0, 20.0),
    blur: 15.0,
    spread: 0.0,
    color: Color32::from_black_alpha(96),
};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, Self::setup)
            .init_resource::<WidgetData>();
    }
}

#[derive(Resource, Default, Debug)]
pub struct WidgetData {
    pub unit_container: HashMap<Faction, UnitContainerData>,
    pub notifications: NotificationsData,
}

impl UiPlugin {
    fn setup(world: &mut World) {
        let Some(ctx) = egui_context(world) else {
            return;
        };
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
                ..default()
            };
            style.spacing.window_margin = Margin::same(13.0);
            style.spacing.slider_rail_height = 2.0;
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
                    bg_fill: VISIBLE_LIGHT, // checkbox background
                    bg_stroke: Default::default(),
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
                    expansion: 1.0,
                },
                active: WidgetVisuals {
                    weak_bg_fill: Color32::TRANSPARENT,
                    bg_fill: Color32::from_gray(55),
                    bg_stroke: Stroke::new(1.0, YELLOW),
                    fg_stroke: Stroke::new(2.0, YELLOW),
                    rounding: Rounding::same(13.0),
                    expansion: 1.0,
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
}

lazy_static! {
    static ref PATH: Id = Id::new("widget_path");
    static ref DRAGGED_PATH: Id = Id::new("dragged_path");
    static ref DRAG_FINISHED_DATA: Id = Id::new("drag_finished");
    static ref HOVERED_PATH: Id = Id::new("hovered_path");
}

pub trait CtxExt {
    fn is_name_enabled(&self, name: &str) -> bool;
    fn is_path_enabled(&self, path: &str) -> bool;
    fn cur_enabled(&self) -> bool;
    fn set_name_enabled(&self, name: &str, v: bool);
    fn set_path_enabled(&self, path: &str, v: bool);
    fn flip_name_enabled(&self, name: &str);
    fn flip_path_enabled(&self, path: &str);
    fn path(&self) -> String;
    fn path_with(&self, name: &str) -> String;
    fn add_path(&self, name: &str);
    fn remove_path(&self);
    fn drag_start(&self, pos: Pos2);
    fn drag_end(&self);
    fn get_dragged(&self) -> Option<(String, Pos2)>;
    fn drag_finished(&self) -> Option<(String, String)>;
    fn set_hovered(&self, rect: Rect);
    fn get_hovered(&self) -> Option<(String, Rect)>;
}

impl CtxExt for egui::Context {
    fn is_name_enabled(&self, name: &str) -> bool {
        self.data(|r| r.get_temp(self.path_with(name).into()))
            .unwrap_or_default()
    }
    fn is_path_enabled(&self, path: &str) -> bool {
        self.data(|r| r.get_temp(Id::new(path))).unwrap_or_default()
    }
    fn cur_enabled(&self) -> bool {
        self.is_path_enabled(&self.path())
    }
    fn set_name_enabled(&self, name: &str, v: bool) {
        let p = self.path_with(name);
        self.set_path_enabled(&p, v)
    }
    fn set_path_enabled(&self, path: &str, v: bool) {
        self.data_mut(|w| w.insert_temp(Id::new(path), v))
    }
    fn flip_name_enabled(&self, name: &str) {
        let p = self.path_with(name);
        self.flip_path_enabled(&p)
    }
    fn flip_path_enabled(&self, path: &str) {
        let v = self.is_path_enabled(&path);
        self.data_mut(|w| w.insert_temp(Id::new(path), !v))
    }
    fn path(&self) -> String {
        self.data(|r| r.get_temp(*PATH)).unwrap_or_default()
    }
    fn path_with(&self, name: &str) -> String {
        let p = self.path();
        format!("{p}/{name}")
    }
    fn add_path(&self, name: &str) {
        let mut p = self.path();
        p.push('/');
        p.push_str(name);
        self.data_mut(|w| w.insert_temp(*PATH, p));
    }
    fn remove_path(&self) {
        let mut p = self.path();
        if let Some(pos) = p.rfind('/') {
            let _ = p.split_off(pos);
            self.data_mut(|w| w.insert_temp(*PATH, p));
        }
    }
    fn drag_start(&self, pos: Pos2) {
        let p = self.path();
        self.data_mut(|w| w.insert_temp(*DRAGGED_PATH, (p, pos)));
    }
    fn drag_end(&self) {
        if let Some((from, _)) = self.data_mut(|w| w.remove_temp::<(String, Pos2)>(*DRAGGED_PATH)) {
            if let Some((to, rect)) = self.get_hovered() {
                if !from.eq(&to) && self.pointer_latest_pos().is_some_and(|p| rect.contains(p)) {
                    self.data_mut(|w| w.insert_temp(*DRAG_FINISHED_DATA, (from, to)));
                }
            }
        }
    }
    fn drag_finished(&self) -> Option<(String, String)> {
        self.data_mut(|w| w.remove_temp::<(String, String)>(*DRAG_FINISHED_DATA))
    }
    fn get_dragged(&self) -> Option<(String, Pos2)> {
        self.data(|r| r.get_temp::<(String, Pos2)>(*DRAGGED_PATH))
    }
    fn set_hovered(&self, rect: Rect) {
        let p = self.path();
        self.data_mut(|w| w.insert_temp(*HOVERED_PATH, (p, rect)))
    }
    fn get_hovered(&self) -> Option<(String, Rect)> {
        self.data(|r| r.get_temp::<(String, Rect)>(*HOVERED_PATH))
    }
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
            Side::Right | Side::Left => false,
            Side::Top | Side::Bottom => true,
        }
    }
}
