use bevy_egui::egui::epaint::text::{FontInsert, FontPriority, InsertFontFamily};

use super::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, Self::ui)
            .add_systems(Startup, setup_ui);
    }
}

impl UiPlugin {
    fn ui(world: &mut World) {
        let Some(ctx) = &egui_context(world) else {
            return;
        };
        TopBottomPanel::top("top_bar").show(ctx, |ui| {
            TopBar::ui(ui, world);
        });
        TilePlugin::ui(ctx, world);
        Confirmation::show_current(ctx, world);
        NotificationsPlugin::ui(ctx, world);
    }
}
fn setup_ui(mut ctx: Query<&mut EguiContext>) {
    let ctx = ctx.single_mut().into_inner().get_mut();
    ctx.add_font(FontInsert::new(
        "mono",
        FontData::from_static(include_bytes!(
            "../../assets/fonts/SometypeMono-Regular.ttf"
        )),
        [
            InsertFontFamily {
                family: FontFamily::Monospace,
                priority: FontPriority::Highest,
            },
            InsertFontFamily {
                family: FontFamily::Proportional,
                priority: FontPriority::Highest,
            },
            InsertFontFamily {
                family: FontFamily::Name("bold".into()),
                priority: FontPriority::Highest,
            },
        ]
        .into(),
    ));
    ctx.add_font(FontInsert::new(
        "emoji",
        FontData::from_static(include_bytes!(
            "../../assets/fonts/NotoEmoji-VariableFont_wght.ttf"
        )),
        [
            InsertFontFamily {
                family: FontFamily::Monospace,
                priority: FontPriority::Lowest,
            },
            InsertFontFamily {
                family: FontFamily::Proportional,
                priority: FontPriority::Lowest,
            },
            InsertFontFamily {
                family: FontFamily::Name("bold".into()),
                priority: FontPriority::Lowest,
            },
        ]
        .into(),
    ));
    ctx.style_mut(|style| {
        style.text_styles = [
            (
                TextStyle::Heading,
                FontId::new(26.0, FontFamily::Name("bold".into())),
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
        style.debug.show_unaligned = false;
        style.animation_time = 0.1;
        style.spacing = Spacing {
            item_spacing: egui::vec2(8.0, 6.0),
            button_padding: egui::vec2(3.0, 3.0),
            combo_width: 10.0,
            interact_size: egui::vec2(20.0, 20.0),
            ..Default::default()
        };
        style.wrap_mode = Some(egui::TextWrapMode::Extend);
        style.spacing.window_margin = Margin::same(8);
        style.spacing.slider_rail_height = 2.0;
        style.spacing.button_padding = egui::vec2(8.0, 2.0);
        style.visuals.panel_fill = Color32::from_rgb(0, 0, 0);
        style.visuals.striped = false;
        style.visuals.slider_trailing_fill = true;
        style.visuals.faint_bg_color = BG_LIGHT;
        style.visuals.handle_shape = HandleShape::Rect { aspect_ratio: 0.1 };
        style.visuals.selection.bg_fill = YELLOW_DARK;
        style.visuals.resize_corner_size = 0.0;
        style.visuals.window_stroke = STROKE_DARK;
        style.visuals.window_fill = EMPTINESS;
        style.visuals.extreme_bg_color = TRANSPARENT;
        style.visuals.widgets = Widgets {
            noninteractive: WidgetVisuals {
                weak_bg_fill: Color32::TRANSPARENT,
                bg_fill: Color32::from_gray(27),
                bg_stroke: STROKE_BG_DARK, // separators, indentation lines
                fg_stroke: Stroke::new(1.0, VISIBLE_DARK), // normal text color
                corner_radius: CornerRadius::same(13),
                expansion: 0.0,
            },
            inactive: WidgetVisuals {
                weak_bg_fill: Color32::TRANSPARENT,
                bg_fill: BG_LIGHT, // checkbox background
                bg_stroke: STROKE_BG_DARK,
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
                bg_stroke: STROKE_BG_DARK,
                fg_stroke: Stroke::new(1.0, Color32::from_gray(210)),
                corner_radius: CornerRadius::same(13),
                expansion: 0.0,
            },
        };
    });
}
