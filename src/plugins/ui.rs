use bevy_egui::egui::{
    style::WidgetVisuals, FontData, FontDefinitions, InnerResponse, Layout, Rounding, Stroke,
    TextStyle, Vec2, Widget,
};

use super::*;

fn light_gray() -> Color32 {
    hex_color!("#6F6F6F")
}
fn dark_gray() -> Color32 {
    hex_color!("#393939")
}
fn black() -> Color32 {
    hex_color!("#000000")
}
fn white() -> Color32 {
    hex_color!("#ffffff")
}
fn yellow() -> Color32 {
    hex_color!("#D98F00")
}
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, Self::setup)
            .add_systems(Update, Self::ui);
    }
}

impl UiPlugin {
    fn setup(world: &mut World) {
        let ctx = egui_context(world);
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
        ctx.set_fonts(fonts);

        ctx.style_mut(|style| {
            style.text_styles = [
                (TextStyle::Heading, FontId::new(26.0, FontFamily::Monospace)),
                (
                    TextStyle::Name("Heading2".into()),
                    FontId::new(22.0, FontFamily::Monospace),
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
            style.visuals.window_fill = black();
            style.visuals.window_stroke.color = white();
            style.visuals.window_stroke.width = 1.0;
            style.spacing.window_margin = 8.0.into();
            // dbg!(style.visuals.widgets.hovered);
            style.visuals.widgets.inactive = WidgetVisuals {
                bg_fill: white(),
                weak_bg_fill: white(),
                bg_stroke: Stroke::NONE,
                rounding: Rounding::same(2.0),
                fg_stroke: Stroke::new(1.0, black()),
                expansion: 0.0,
            };
            style.visuals.widgets.hovered = WidgetVisuals {
                bg_fill: yellow(),
                weak_bg_fill: yellow(),
                bg_stroke: Stroke::NONE,
                rounding: Rounding::same(2.0),
                fg_stroke: Stroke::new(1.0, white()),
                expansion: 0.0,
            };
            style.visuals.widgets.active = WidgetVisuals {
                bg_fill: white(),
                weak_bg_fill: white(),
                bg_stroke: Stroke::new(1.0, yellow()),
                rounding: Rounding::same(2.0),
                fg_stroke: Stroke::new(1.0, yellow()),
                expansion: 0.0,
            };
            // style.spacing.item_spacing = [12.0, 2.0].into();
        });
    }

    fn ui(world: &mut World) {
        let ctx = &egui_context(world);
        Window::new(RichText::new("Main Menu").size(14.0).color(white()))
            .min_width(100.0)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.vertical_centered_justified(|ui| {
                    // Frame::none().fill(white()).show(ui, |ui| {
                    //     ui.with_layout(
                    //         Layout::left_to_right(egui::Align::Min)
                    //             .with_main_align(egui::Align::Min)
                    //             .with_main_justify(true),
                    //         |ui| ui.colored_label(black(), "Main Menu"),
                    //     )
                    // });
                    frame(ui, |ui| line(ui, [Button::new("Continue")].into()));
                    frame(ui, |ui| {
                        line(
                            ui,
                            [Button::new("Random Ladder"), Button::new("New Ladder")].into(),
                        )
                    });
                    frame(ui, |ui| line(ui, [Button::new("Exit")].into()));
                })
            });
    }
}

fn frame<R>(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
    Frame::none()
        .stroke(Stroke::new(1.0, dark_gray()))
        .inner_margin(6.0)
        .show(ui, |ui| {
            ui.with_layout(Layout::left_to_right(egui::Align::Min), add_contents)
                .inner
        })
}

fn line(ui: &mut Ui, widgets: Vec<impl Widget>) {
    let columns = widgets.len() as f32;
    let w = ui.available_width() / columns - ui.style().spacing.item_spacing.x;
    for widget in widgets {
        ui.add_sized([w, 24.0], widget);
    }
}
