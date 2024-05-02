use bevy_egui::egui::{Id, Image, Order, Sense};

use super::*;

pub fn light_gray() -> Color32 {
    hex_color!("#6F6F6F")
}
pub fn dark_gray() -> Color32 {
    hex_color!("#393939")
}
pub fn black() -> Color32 {
    hex_color!("#000000")
}
pub fn light_black() -> Color32 {
    hex_color!("#202020")
}
pub fn white() -> Color32 {
    hex_color!("#ffffff")
}
pub fn yellow() -> Color32 {
    hex_color!("#D98F00")
}
pub fn orange() -> Color32 {
    hex_color!("#FC4C02")
}
pub fn red() -> Color32 {
    hex_color!("#E53935")
}
pub fn green() -> Color32 {
    hex_color!("#64DD17")
}
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, Self::setup);
    }
}

impl UiPlugin {
    fn setup(world: &mut World) {
        let ctx = &if let Some(context) = egui_context(world) {
            context
        } else {
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
            style.visuals.window_fill = black();
            style.visuals.window_stroke.color = white();
            style.visuals.window_stroke.width = 1.0;
            // style.spacing.window_margin = 8.0.into();
            style.spacing.window_margin = 0.0.into();
            style.spacing.slider_width = 200.0;
            // dbg!(style.visuals.widgets.hovered);
            style.visuals.widgets.inactive = WidgetVisuals {
                bg_fill: black(),
                weak_bg_fill: black(),
                bg_stroke: Stroke::new(1.0, white()),
                rounding: Rounding::same(0.0),
                fg_stroke: Stroke::new(1.0, white()),
                expansion: 0.0,
            };
            style.visuals.widgets.active = WidgetVisuals {
                bg_fill: yellow(),
                weak_bg_fill: yellow(),
                bg_stroke: Stroke::NONE,
                rounding: Rounding::same(0.0),
                fg_stroke: Stroke::new(1.0, white()),
                expansion: 0.0,
            };
            style.visuals.widgets.hovered = WidgetVisuals {
                bg_fill: black(),
                weak_bg_fill: black(),
                bg_stroke: Stroke::new(1.0, yellow()),
                rounding: Rounding::same(0.0),
                fg_stroke: Stroke::new(1.0, yellow()),
                expansion: 1.0,
            };
            style.visuals.widgets.noninteractive = WidgetVisuals {
                bg_fill: black(),
                weak_bg_fill: black(),
                bg_stroke: Stroke::NONE,
                rounding: Rounding::same(0.0),
                fg_stroke: Stroke::new(1.0, light_gray()),
                expansion: 0.0,
            };
            style.visuals.faint_bg_color = light_black();
            // style.spacing.item_spacing = [12.0, 2.0].into();
        });
    }
}

pub fn text_dots_text(text1: &ColoredString, text2: &ColoredString, ui: &mut Ui) {
    ui.horizontal(|ui| {
        let rect = ui.max_rect();
        let left = rect.left() + text1.label(ui).rect.width() + 3.0;
        let right = rect.right()
            - 3.0
            - ui.with_layout(Layout::right_to_left(egui::Align::Min), |ui| {
                text2.label(ui);
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

pub struct GameWindow<'a> {
    area: Area,
    frame: Option<Frame>,
    title: &'a str,
    title_bar: bool,
    stroke: bool,
    min_width: f32,
    max_width: f32,
    color: Option<Color32>,
    close_action: Option<StoredAction>,
}

impl GameWindow<'_> {
    pub fn show<R>(
        self,
        ctx: &egui::Context,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        self.area
            .show(ctx, |ui| self.show_ui(ui, add_contents))
            .inner
    }
    pub fn show_ui<R>(
        mut self,
        ui: &mut Ui,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        if !self.stroke {
            self.frame = Some(Frame::none());
        }
        let style = ui.style();
        let mut stroke = style.visuals.window_stroke;
        stroke.color = self.color.unwrap_or(style.visuals.window_stroke.color);
        self.frame
            .unwrap_or(Frame::window(style).stroke(stroke))
            .show(ui, |ui| {
                ui.set_min_width(self.min_width);
                ui.set_max_width(self.max_width);
                let id = ui.id();
                if self.title_bar {
                    let v = &ui.style().visuals.clone();
                    let mut rounding = v.window_rounding;
                    rounding.se = 0.0;
                    rounding.sw = 0.0;
                    Frame::none()
                        .fill(stroke.color)
                        .rounding(rounding)
                        .stroke(stroke)
                        .show(ui, |ui| {
                            if let Some(w) = ui.data_mut(|w| w.get_temp::<f32>(id)) {
                                ui.set_width(w);
                            }
                            ui.with_layout(
                                Layout::top_down(egui::Align::Min).with_cross_justify(true),
                                |ui| {
                                    Frame::none()
                                        .inner_margin(Margin::symmetric(8.0, 0.0))
                                        .show(ui, |ui| {
                                            ui.horizontal(|ui| {
                                                Label::new(
                                                    RichText::new(self.title)
                                                        .text_style(TextStyle::Heading)
                                                        .size(15.0)
                                                        .color(black()),
                                                )
                                                .wrap(false)
                                                .selectable(false)
                                                .ui(ui);
                                                if let Some(action) = self.close_action {
                                                    ui.with_layout(
                                                        Layout::right_to_left(egui::Align::Center),
                                                        |ui| {
                                                            if " x "
                                                                .add_color(black())
                                                                .as_label(ui)
                                                                .selectable(false)
                                                                .sense(Sense::click())
                                                                .ui(ui)
                                                                .clicked()
                                                            {
                                                                OperationsPlugin::add(|w| {
                                                                    action(w)
                                                                });
                                                            }
                                                        },
                                                    );
                                                }
                                            });
                                        })
                                },
                            );
                        });
                }
                let resp = add_contents(ui);
                if self.title_bar {
                    ui.data_mut(|w| w.insert_temp(id, ui.available_width()));
                }
                resp
            })
    }
    pub fn default_pos_vec(mut self, pos: Vec2) -> Self {
        self.area = self.area.default_pos(pos2(pos.x, pos.y));
        self
    }
    pub fn default_pos(mut self, pos: Pos2) -> Self {
        self.area = self.area.default_pos(pos);
        self
    }
    pub fn fixed_pos(mut self, pos: Vec2) -> Self {
        self.area = self.area.fixed_pos(pos2(pos.x, pos.y));
        self
    }
    pub fn id(mut self, id: impl std::hash::Hash) -> Self {
        self.area = self.area.id(Id::new(id).with(self.title));
        self
    }
    pub fn set_width(mut self, width: f32) -> Self {
        self.min_width = width;
        self.max_width = width;
        self
    }
    pub fn set_max_width(mut self, width: f32) -> Self {
        self.max_width = width;
        self
    }
    pub fn set_min_width(mut self, width: f32) -> Self {
        self.min_width = width;
        self
    }
    pub fn anchor(mut self, align: Align2, offset: impl Into<egui::Vec2>) -> Self {
        self.area = self.area.anchor(align, offset);
        self
    }
    pub fn title_bar(mut self, enable: bool) -> Self {
        self.title_bar = enable;
        self
    }
    pub fn stroke(mut self, enable: bool) -> Self {
        self.stroke = enable;
        self
    }
    pub fn set_color(mut self, color: Color32) -> Self {
        self.color = Some(color);
        self
    }
    pub fn order(mut self, order: Order) -> Self {
        self.area = self.area.order(order);
        self
    }
    pub fn entity_anchor(
        mut self,
        entity: Entity,
        pivot: Align2,
        offset: Vec2,
        world: &World,
    ) -> Self {
        let pos = entity_screen_pos(entity, offset, world);
        self.area = self.area.fixed_pos([pos.x, pos.y]).pivot(pivot);
        self
    }
    pub fn set_close_action(mut self, action: StoredAction) -> Self {
        self.close_action = Some(action);
        self
    }
}

pub fn window(title: &str) -> GameWindow<'_> {
    GameWindow {
        area: Area::new(title.to_owned().into())
            .constrain(true)
            .pivot(Align2::CENTER_CENTER),
        title,
        title_bar: true,
        stroke: true,
        color: None,
        min_width: 250.0,
        max_width: 250.0,
        frame: None,
        close_action: None,
    }
}

pub fn frame<R>(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
    Frame::none()
        .stroke(Stroke::new(1.0, dark_gray()))
        .inner_margin(6.0)
        .outer_margin(6.0)
        .rounding(0.0)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.vertical_centered_justified(add_contents).inner
        })
}

pub fn frame_horizontal<R>(
    ui: &mut Ui,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> InnerResponse<R> {
    Frame::none()
        .stroke(Stroke::new(1.0, dark_gray()))
        .inner_margin(6.0)
        .outer_margin(6.0)
        .rounding(0.0)
        .show(ui, |ui| ui.horizontal_centered(add_contents).inner)
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

pub trait PrimarySecondaryExtensions {
    fn button_color(&mut self, text: impl Into<WidgetText>, color: Color32) -> Response;
    fn button_red(&mut self, text: impl Into<WidgetText>) -> Response;
    fn button_primary(&mut self, text: impl Into<WidgetText>) -> Response;
    fn button_or_primary(&mut self, text: impl Into<WidgetText>, primary: bool) -> Response;
}

impl PrimarySecondaryExtensions for Ui {
    fn button_primary(&mut self, text: impl Into<WidgetText>) -> Response {
        let style = self.style_mut();
        let prev_style = style.clone();
        style.visuals.widgets.inactive.weak_bg_fill = white();
        style.visuals.widgets.hovered.weak_bg_fill = white();
        style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, black());
        let response = Button::new(text).min_size(egui::Vec2::ZERO).ui(self);
        self.set_style(prev_style);
        response
    }

    fn button_or_primary(&mut self, text: impl Into<WidgetText>, primary: bool) -> Response {
        if primary {
            self.button_primary(text)
        } else {
            self.button(text)
        }
    }

    fn button_red(&mut self, text: impl Into<WidgetText>) -> Response {
        self.button_color(text, red())
    }

    fn button_color(&mut self, text: impl Into<WidgetText>, color: Color32) -> Response {
        let prev_style = self.style().clone();
        let visuals = &mut self.style_mut().visuals.widgets.inactive;
        visuals.fg_stroke.color = color;
        visuals.bg_stroke.color = color;
        let r = self.button(text);
        self.set_style(prev_style);
        r
    }
}

#[derive(Clone, Copy)]
pub enum Icon {
    Lightning,
    Target,
    Flame,
    Discord,
    Youtube,
    Itch,
    Github,
}

impl Icon {
    pub fn show(self, ui: &mut Ui) {
        self.image().fit_to_original_size(1.0).ui(ui);
    }

    pub fn image(self) -> Image<'static> {
        match self {
            Icon::Lightning => {
                egui::Image::new(egui::include_image!("../../assets/svg/lightning.svg"))
                    .fit_to_original_size(1.0)
            }
            Icon::Target => egui::Image::new(egui::include_image!("../../assets/svg/target.svg"))
                .fit_to_original_size(1.0),
            Icon::Flame => egui::Image::new(egui::include_image!("../../assets/svg/flame.svg"))
                .fit_to_original_size(1.0),
            Icon::Discord => egui::Image::new(egui::include_image!("../../assets/svg/discord.svg"))
                .fit_to_original_size(0.8),
            Icon::Youtube => egui::Image::new(egui::include_image!("../../assets/svg/youtube.svg"))
                .fit_to_original_size(0.8),
            Icon::Itch => egui::Image::new(egui::include_image!("../../assets/svg/itch.svg"))
                .fit_to_original_size(0.8),
            Icon::Github => egui::Image::new(egui::include_image!("../../assets/svg/github.svg"))
                .fit_to_original_size(0.8),
        }
    }
}
