use std::str::FromStr;

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
            // .add_systems(Update, Self::ui)
            ;
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
                bg_stroke: Stroke::new(1.0, white()),
                rounding: Rounding::same(0.0),
                fg_stroke: Stroke::new(1.0, black()),
                expansion: 0.0,
            };
            style.visuals.widgets.active = WidgetVisuals {
                bg_fill: yellow(),
                weak_bg_fill: yellow(),
                bg_stroke: Stroke::NONE,
                rounding: Rounding::same(0.0),
                fg_stroke: Stroke::new(1.0, white()),
                expansion: 2.0,
            };
            style.visuals.widgets.hovered = WidgetVisuals {
                bg_fill: white(),
                weak_bg_fill: white(),
                bg_stroke: Stroke::new(1.0, yellow()),
                rounding: Rounding::same(0.0),
                fg_stroke: Stroke::new(1.0, yellow()),
                expansion: 0.0,
            };
            style.visuals.widgets.noninteractive = WidgetVisuals {
                bg_fill: black(),
                weak_bg_fill: black(),
                bg_stroke: Stroke::NONE,
                rounding: Rounding::same(0.0),
                fg_stroke: Stroke::new(1.0, light_gray()),
                expansion: 0.0,
            };
            // style.spacing.item_spacing = [12.0, 2.0].into();
        });
    }

    fn ui(world: &mut World) {
        let ctx = &egui_context(world);
        if let Some(unit) = Pools::try_get(world)
            .and_then(|p| p.heroes.get("Priest"))
            .cloned()
        {
            show_unit_card(&unit, world);
        }

        window("MAIN MENU").show(ctx, |ui| {
            frame(ui, |ui| {
                ui.set_enabled(false);
                ui.button("CONTINUE");
            });
            frame(ui, |ui| {
                ui.columns(2, |ui| {
                    ui[0].vertical_centered_justified(|ui| {
                        ui.add(Button::new("RANDOM LADDER"));
                    });
                    ui[1].vertical_centered_justified(|ui| {
                        ui.add(Button::new("NEW LADDER"));
                    });
                });
            });
            frame(ui, |ui| ui.button("EXIT"));
        });
        window("TEXT").show( ctx, |ui| {
            frame(ui, |ui| {
                Label::new(RichText::new("Heading").heading().color(white())).ui(ui);
                ui.label("Lorem ipsum dolor sit amet consectetur adipiscing elit Ut et massa mi. Aliquam in hendrerit.");
            });
        });
        window("UNIT").show(ctx, |ui| {
            frame(ui, |ui| {
                Label::new(
                    RichText::new("Priest")
                        .heading()
                        .color(hex_color!("#FFF176")),
                )
                .ui(ui);

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
                    let line = egui::Shape::dotted_line(
                        &[[left, bottom].into(), [right, bottom].into()],
                        light_gray(),
                        8.0,
                        0.5,
                    );
                    ui.painter().add(line);
                });
            });
        });
    }
}

pub fn show_unit_card(unit: &PackedUnit, world: &mut World) {
    let house_color = Pools::get_house_color(&unit.house, world)
        .unwrap_or(Color::WHITE)
        .c32();
    let ctx = &egui_context(world);
    window("unit").show(ctx, |ui| {
        frame(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading(RichText::new(&unit.name).color(house_color));
            });
            ui.label(&unit.description);
            text_dots_text("hp", &unit.hp.to_string(), ui);
            text_dots_text("atk", &unit.atk.to_string(), ui);
        });
        if !unit.statuses.is_empty() {
            frame(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading(RichText::new("Statuses").color(white()));
                });
                for (name, charges) in unit.statuses.iter() {
                    let c = Pools::get_status_house(name, world).color.clone().into();
                    text_dots_text_colored((name, c), (&charges.to_string(), white()), ui);
                    if let Some(status) = Pools::get_status(name, world) {
                        let state = VarState::new_with(VarName::Charges, VarValue::Int(*charges));
                        ui.label(status.description.inject_vars(&state));
                    }
                }
            });
        }
    });
}

pub fn text_dots_text_colored(text1: (&str, Color32), text2: (&str, Color32), ui: &mut Ui) {
    ui.horizontal(|ui| {
        let rect = ui.max_rect();
        let left = rect.left()
            + ui.add(Label::new(RichText::new(text1.0).color(text1.1)))
                .rect
                .width()
            + 3.0;
        let right = rect.right()
            - 3.0
            - ui.with_layout(Layout::right_to_left(egui::Align::Min), |ui| {
                ui.colored_label(white(), RichText::new(text2.0).color(text2.1));
            })
            .response
            .rect
            .width();
        let bottom = rect.bottom() - 6.0;
        let line = egui::Shape::dotted_line(
            &[[left, bottom].into(), [right, bottom].into()],
            light_gray(),
            8.0,
            0.5,
        );
        ui.painter().add(line);
    });
}

pub fn text_dots_text(text1: &str, text2: &str, ui: &mut Ui) {
    text_dots_text_colored((text1, light_gray()), (text2, white()), ui);
}

pub struct GameWindow<'a>(pub Window<'a>, &'a str);

impl GameWindow<'_> {
    pub fn show(self, ctx: &egui::Context, add_contents: impl FnOnce(&mut Ui)) {
        self.0.show(ctx, |ui| {
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
                                        RichText::new(self.1)
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
        // self.0.show(ctx, add_contents);
    }
}

pub fn window<'a>(title: &'a str) -> GameWindow<'a> {
    GameWindow(
        Window::new(title)
            .min_width(80.0)
            .title_bar(false)
            .collapsible(false),
        title,
    )
}

pub fn frame<R>(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
    Frame::none()
        .stroke(Stroke::new(1.0, dark_gray()))
        .inner_margin(6.0)
        .outer_margin(6.0)
        .rounding(0.0)
        .show(ui, |ui| ui.vertical_centered_justified(add_contents).inner)
}

pub trait IntoC32 {
    fn c32(&self) -> Color32;
}

impl IntoC32 for Color {
    fn c32(&self) -> Color32 {
        let a = self.as_rgba_u8();
        Color32::from_rgba_unmultiplied(a[0], a[1], a[2], a[3])
    }
}

pub trait InjectVars {
    fn inject_vars(&self, state: &VarState) -> WidgetText;
}

impl InjectVars for str {
    fn inject_vars(&self, state: &VarState) -> WidgetText {
        let mut job = LayoutJob::default();
        for (mut line, bracketed) in str_extract_brackets(self, ("{", "}")) {
            let color = if bracketed { white() } else { light_gray() };
            if let Ok(var) = VarName::from_str(&line) {
                if let Ok(value) = state.get_string(var) {
                    line = value;
                }
            }
            job.append(&line, 0.0, TextFormat { color, ..default() });
        }

        WidgetText::LayoutJob(job)
    }
}

fn str_extract_brackets(mut source: &str, pattern: (&str, &str)) -> Vec<(String, bool)> {
    let mut lines: Vec<(String, bool)> = default();
    while let Some(opening) = source.find(pattern.0) {
        let left = &source[..opening];
        let closing = source.find(pattern.1).unwrap();
        let mid = &source[opening + 1..closing];
        lines.push((left.to_owned(), false));
        lines.push((mid.to_owned(), true));
        source = &source[closing + 1..];
    }
    lines.push((source.to_owned(), false));
    lines
}

pub trait ButtonExtensions {
    fn button_secondary(&mut self, text: impl Into<WidgetText>) -> Response;
}

impl ButtonExtensions for Ui {
    fn button_secondary(&mut self, text: impl Into<WidgetText>) -> Response {
        let style = self.style_mut();
        style.visuals.widgets.inactive.weak_bg_fill = black();
        style.visuals.widgets.hovered.weak_bg_fill = black();
        style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, white());
        Button::new(text).min_size(egui::Vec2::ZERO).ui(self)
    }
}
