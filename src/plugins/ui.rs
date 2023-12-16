use bevy_egui::egui::Id;

use super::*;

pub fn light_gray() -> Color32 {
    hex_color!("#6F6F6F")
}
fn dark_gray() -> Color32 {
    hex_color!("#393939")
}
fn black() -> Color32 {
    hex_color!("#000000")
}
pub fn white() -> Color32 {
    hex_color!("#ffffff")
}
pub fn yellow() -> Color32 {
    hex_color!("#D98F00")
}
pub fn red() -> Color32 {
    hex_color!("#E53935")
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
                expansion: 2.0,
            };
            style.visuals.widgets.hovered = WidgetVisuals {
                bg_fill: black(),
                weak_bg_fill: black(),
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
}

pub fn text_dots_text(text1: &ColoredString, text2: &ColoredString, ui: &mut Ui) {
    ui.horizontal(|ui| {
        let rect = ui.max_rect();
        let left = rect.left() + ui.add(Label::new(text1.widget())).rect.width() + 3.0;
        let right = rect.right()
            - 3.0
            - ui.with_layout(Layout::right_to_left(egui::Align::Min), |ui| {
                ui.label(text2.widget());
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
    window: Window<'a>,
    title: &'a str,
    title_bar: bool,
    stroke: bool,
}

impl GameWindow<'_> {
    pub fn show(mut self, ctx: &egui::Context, add_contents: impl FnOnce(&mut Ui)) {
        if !self.stroke {
            self.window = self.window.frame(Frame::none());
        }
        self.window.show(ctx, |ui| {
            if self.title_bar {
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
                                            RichText::new(self.title)
                                                .text_style(TextStyle::Heading)
                                                .size(15.0)
                                                .color(black()),
                                        );
                                    })
                            },
                        );
                    });
            }
            add_contents(ui)
        });
    }
    pub fn default_pos(mut self, pos: Vec2) -> Self {
        self.window = self.window.default_pos(pos2(pos.x, pos.y));
        self
    }
    pub fn fixed_pos(mut self, pos: Vec2) -> Self {
        self.window = self.window.fixed_pos(pos2(pos.x, pos.y));
        self
    }
    pub fn id(mut self, id: impl std::hash::Hash) -> Self {
        self.window = self.window.id(Id::new(id).with(self.title));
        self
    }
    pub fn set_width(mut self, width: f32) -> Self {
        self.window = self.window.default_width(width);
        self
    }
    pub fn anchor(mut self, align: Align2, offset: impl Into<egui::Vec2>) -> Self {
        self.window = self.window.anchor(align, offset);
        self
    }
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.window = self.window.resizable(resizable);
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
    pub fn entity_anchor(
        mut self,
        entity: Entity,
        pivot: Align2,
        offset: Vec2,
        world: &mut World,
    ) -> Self {
        let pos = entity_screen_pos(entity, world) + offset;
        self.window = self.window.fixed_pos([pos.x, pos.y]).pivot(pivot);
        self
    }
}

pub fn window(title: &str) -> GameWindow<'_> {
    GameWindow {
        window: Window::new(title)
            .default_width(300.0)
            .title_bar(false)
            .collapsible(false),
        title,
        title_bar: true,
        stroke: true,
    }
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

pub trait PrimarySecondaryExtensions {
    fn button_primary(&mut self, text: impl Into<WidgetText>) -> Response;
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
}
