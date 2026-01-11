use bevy_egui::{
    EguiFullOutput, EguiInput,
    egui::{
        LayerId,
        epaint::text::{FontInsert, FontPriority, InsertFontFamily},
    },
};

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
        let (ctx, mut egui_input) = world
            .query::<(&mut EguiContext, &mut EguiInput)>()
            .single_mut(world)
            .unwrap();
        let ctx = ctx.get().clone();
        ctx.begin_pass(egui_input.take().0);
        TopBottomPanel::top("top_bar").show(&ctx, |ui| {
            ui.scope_builder(
                UiBuilder::new().layer_id(LayerId::new(Order::Foreground, Id::new("settings"))),
                |ui| {
                    TopBar::ui(ui, world);
                },
            );
        });
        TilePlugin::ui(&ctx, world);
        WindowPlugin::show_all(&ctx, world);
        Confirmation::show_current(&ctx, world);
        NotificationsPlugin::ui(&ctx, world);
        let new_output = ctx.end_pass();
        let mut output = world
            .query::<&mut EguiFullOutput>()
            .single_mut(world)
            .unwrap();
        **output = Some(new_output);
    }
}
fn setup_ui(mut ctx: Query<&mut EguiContext>) {
    let ctx = ctx.single_mut().unwrap().get().clone();

    pd_mut(|d| {
        let colorix = &mut d.client_settings.theme;
        colorix.generate_scale();
        colorix.apply(&ctx);
        colorix.clone().save();
    });
    init_style_map(&colorix());

    ctx.add_font(FontInsert::new(
        "mono_regular",
        FontData::from_static(include_bytes!(
            "../../../assets/fonts/SometypeMono-Regular.ttf"
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
        ]
        .into(),
    ));
    ctx.add_font(FontInsert::new(
        "mono_bold",
        FontData::from_static(include_bytes!(
            "../../../assets/fonts/SometypeMono-Bold.ttf"
        )),
        [InsertFontFamily {
            family: FontFamily::Name("bold".into()),
            priority: FontPriority::Highest,
        }]
        .into(),
    ));
    ctx.add_font(FontInsert::new(
        "heading",
        FontData::from_static(include_bytes!(
            "../../../assets/fonts/Merriweather_24pt-Medium.ttf"
        )),
        [InsertFontFamily {
            family: FontFamily::Name("heading".into()),
            priority: FontPriority::Highest,
        }]
        .into(),
    ));
    ctx.add_font(FontInsert::new(
        "emoji_icon",
        FontData::from_static(epaint_default_fonts::EMOJI_ICON),
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
    // ctx.add_font(FontInsert::new(
    //     "emoji_regular",
    //     FontData::from_static(epaint_default_fonts::NOTO_EMOJI_REGULAR),
    //     [
    //         InsertFontFamily {
    //             family: FontFamily::Monospace,
    //             priority: FontPriority::Lowest,
    //         },
    //         InsertFontFamily {
    //             family: FontFamily::Proportional,
    //             priority: FontPriority::Lowest,
    //         },
    //         InsertFontFamily {
    //             family: FontFamily::Name("bold".into()),
    //             priority: FontPriority::Lowest,
    //         },
    //     ]
    //     .into(),
    // ));
    ctx.add_font(FontInsert::new(
        "emoji_noto",
        FontData::from_static(include_bytes!(
            "../../../assets/fonts/NotoEmoji-VariableFont_wght.ttf"
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
                FontId::new(28.0, FontFamily::Name("heading".into())),
            ),
            (
                TextStyle::Name("Heading2".into()),
                FontId::new(22.0, FontFamily::Name("heading".into())),
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
        #[cfg(debug_assertions)]
        {
            style.debug.show_unaligned = false;
        }
        style.animation_time = 0.1;
        style.spacing = Spacing {
            item_spacing: egui::vec2(8.0, 6.0),
            button_padding: egui::vec2(8.0, 2.0),
            combo_width: 10.0,
            interact_size: egui::vec2(20.0, 20.0),
            ..Default::default()
        };
        style.wrap_mode = Some(egui::TextWrapMode::Extend);
        style.spacing.window_margin = Margin::same(8);
        style.spacing.slider_rail_height = 2.0;
        style.visuals.striped = false;
        style.visuals.slider_trailing_fill = true;
        style.visuals.handle_shape = HandleShape::Rect { aspect_ratio: 0.1 };
    });
}
