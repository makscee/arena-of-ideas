use bevy_egui::egui::{
    style::WidgetVisuals, FontData, FontDefinitions, InnerResponse, Layout, Painter, Response,
    Rounding, Shape, Stroke, TextStyle, Vec2, Widget,
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
        fonts
            .families
            .get_mut(&FontFamily::Proportional)
            .unwrap()
            .insert(0, "bold".to_owned());
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
            // style.spacing.window_margin = 8.0.into();
            style.spacing.window_margin = 0.0.into();
            // dbg!(style.visuals.widgets.hovered);
            style.visuals.widgets.inactive = WidgetVisuals {
                bg_fill: white(),
                weak_bg_fill: white(),
                bg_stroke: Stroke::NONE,
                rounding: Rounding::same(3.0),
                fg_stroke: Stroke::new(1.0, black()),
                expansion: 0.0,
            };
            style.visuals.widgets.hovered = WidgetVisuals {
                bg_fill: yellow(),
                weak_bg_fill: yellow(),
                bg_stroke: Stroke::NONE,
                rounding: Rounding::same(3.0),
                fg_stroke: Stroke::new(1.0, white()),
                expansion: 0.0,
            };
            style.visuals.widgets.active = WidgetVisuals {
                bg_fill: white(),
                weak_bg_fill: white(),
                bg_stroke: Stroke::new(1.0, yellow()),
                rounding: Rounding::same(3.0),
                fg_stroke: Stroke::new(1.0, yellow()),
                expansion: 0.0,
            };
            style.visuals.widgets.noninteractive = WidgetVisuals {
                bg_fill: black(),
                weak_bg_fill: black(),
                bg_stroke: Stroke::NONE,
                rounding: Rounding::same(3.0),
                fg_stroke: Stroke::new(1.0, light_gray()),
                expansion: 0.0,
            };
            // style.spacing.item_spacing = [12.0, 2.0].into();
        });
    }

    fn ui(world: &mut World) {
        window("MAIN MENU", world, |ui| {
            frame(ui, |ui| {
                let lb = LineBuilder::start(1, ui);
                ui.set_enabled(false);
                lb.show(ui, |ui| {
                    lb.add(Button::new("CONTINUE"), ui);
                });
            });
            frame(ui, |ui| {
                let lb = LineBuilder::start(2, ui);
                lb.show(ui, |ui| {
                    lb.add(Button::new("RANDOM LADDER"), ui);
                    lb.add(Button::new("NEW LADDER"), ui);
                });
            });
            frame(ui, |ui| {
                let lb = LineBuilder::start(1, ui);
                ui.set_enabled(false);
                lb.show(ui, |ui| {
                    lb.add(Button::new("EXIT"), ui);
                });
            });
        });
        window("TEXT", world, |ui| {
            frame(ui, |ui| {
                ui.vertical_centered(|ui| {
                    Label::new(RichText::new("Heading").heading().color(white())).ui(ui);
                });
                ui.label("Lorem ipsum dolor sit amet consectetur adipiscing elit Ut et massa mi. Aliquam in hendrerit.");
            });
        });
        window("UNIT", world, |ui| {
            frame(ui, |ui| {
                ui.vertical_centered(|ui| {
                    Label::new(
                        RichText::new("Priest")
                            .heading()
                            .color(hex_color!("#FFF176")),
                    )
                    .ui(ui);
                });
                ui.label("Battle Start: apply Blessing 2 to all allies");
                ui.horizontal(|ui| {
                    let rect = ui.max_rect();
                    let left = rect.left() + ui.add(Label::new("hp")).rect.width() + 3.0;
                    let right = rect.right()
                        - 3.0
                        - ui.with_layout(Layout::right_to_left(egui::Align::Min), |ui| {
                            ui.colored_label(white(), "3");
                        })
                        .response
                        .rect
                        .width();
                    let bottom = rect.bottom() - 6.0;
                    let line = Shape::dotted_line(
                        &[[left, bottom].into(), [right, bottom].into()],
                        light_gray(),
                        8.0,
                        0.5,
                    );
                    ui.painter().add(line);
                    // ui.painter().hline(
                    //     rect.left()..=rect.right(),
                    //     rect.bottom(),
                    //     Stroke::new(1.0, white()),
                    // );
                });
                // let lb = LineBuilder::start(2, ui);
                // lb.show(ui, |ui| {
                //     // lb.add(Label::new("hp"), ui);
                //     // lb.add(Label::new("3"), ui);
                //     ui.add(Label::new("hp"));
                //     ui.with_layout(Layout::right_to_left(egui::Align::Min), |ui| {
                //         ui.colored_label(white(), "3");
                //     });
                // })
            });
        });
    }
}

fn window<R>(title: &str, world: &mut World, add_contents: impl FnOnce(&mut Ui) -> R) {
    let ctx = &egui_context(world);

    Window::new(title)
        .min_width(100.0)
        .title_bar(false)
        .collapsible(false)
        .show(ctx, |ui| {
            let v = &ui.style().visuals.clone();
            let mut rounding = v.window_rounding;
            rounding.se = 0.0;
            rounding.sw = 0.0;
            Frame::none()
                .fill(white())
                .rounding(rounding)
                .stroke(v.window_stroke)
                .show(ui, |ui| {
                    ui.with_layout(
                        Layout::top_down(egui::Align::Min).with_cross_justify(true),
                        |ui| {
                            Frame::none()
                                .inner_margin(Margin::symmetric(8.0, 0.0))
                                .show(ui, |ui| {
                                    ui.label(
                                        RichText::new(title)
                                            .text_style(TextStyle::Heading)
                                            .size(15.0)
                                            .color(black()),
                                    );
                                })
                        },
                    );
                });
            add_contents(ui)
        });
}

fn frame<R>(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
    Frame::none()
        .stroke(Stroke::new(1.0, dark_gray()))
        .inner_margin(6.0)
        .outer_margin(6.0)
        .show(ui, |ui| add_contents(ui))
}

struct LineBuilder {
    column_width: f32,
    height: f32,
}

impl LineBuilder {
    fn start(columns: usize, ui: &mut Ui) -> Self {
        let width = ui.available_width() / columns as f32 - ui.style().spacing.item_spacing.x;
        Self {
            column_width: width,
            height: 24.0,
        }
    }
    fn show(&self, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui)) {
        ui.with_layout(Layout::left_to_right(egui::Align::Min), add_contents);
    }
    fn add(&self, widget: impl Widget, ui: &mut Ui) -> Response {
        ui.add_sized([self.column_width, self.height], widget)
    }
}
