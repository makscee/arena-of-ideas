use bevy_egui::egui::epaint::text::{FontInsert, FontPriority, InsertFontFamily};
use egui_colors::{tokens::ThemeColor, Theme};

use super::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, Self::ui)
            .add_systems(Startup, (setup_ui, setup_colorix));
    }
}

fn setup_colorix(world: &mut World) {
    let ctx = &egui_context(world).unwrap();
    let theme_main = pd().client_settings.theme;
    let theme_error: Theme = [ThemeColor::Red; 12];
    let theme_success: Theme = [ThemeColor::Green; 12];
    let theme_warning: Theme = [ThemeColor::Orange; 12];
    let theme_info: Theme = [ThemeColor::Cyan; 12];
    let global = egui_colors::Colorix::global(ctx, theme_main);
    let semantics = [
        global,
        egui_colors::Colorix::local_from_style(theme_error, true),
        egui_colors::Colorix::local_from_style(theme_success, true),
        egui_colors::Colorix::local_from_style(theme_warning, true),
        egui_colors::Colorix::local_from_style(theme_info, true),
    ]
    .to_vec();
    let mut colorix = Colorix { semantics };
    world.insert_resource(bevy::render::camera::ClearColor(
        colorix.tokens_global().app_background().to_color(),
    ));
    colorix.apply(ctx);
    colorix.save();
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
        WindowPlugin::show_all(ctx, world);
        Confirmation::show_current(ctx, world);
        NotificationsPlugin::ui(ctx, world);
    }
}
fn setup_ui(mut ctx: Query<&mut EguiContext>) {
    let ctx = ctx.single_mut().into_inner().get_mut();
    ctx.add_font(FontInsert::new(
        "mono_regular",
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
        "mono_bold",
        FontData::from_static(include_bytes!("../../assets/fonts/SometypeMono-Bold.ttf")),
        [InsertFontFamily {
            family: FontFamily::Name("bold".into()),
            priority: FontPriority::Highest,
        }]
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
        style.visuals.striped = false;
        style.visuals.slider_trailing_fill = true;
        style.visuals.handle_shape = HandleShape::Rect { aspect_ratio: 0.1 };
    });
}
