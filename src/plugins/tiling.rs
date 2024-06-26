use super::*;

#[derive(Resource, Default)]
struct Data {
    t: f32,
}

#[derive(Default)]
struct Tile {
    name: &'static str,
    side: Side,
    close_btn: bool,
    title: bool,
    content: Option<Box<dyn FnOnce(&mut Ui, &mut World) + Send + Sync>>,
    child: Option<Box<dyn FnOnce(&mut Ui, &mut World) + Send + Sync>>,
}

impl Tile {
    fn right(name: &'static str) -> Self {
        Self {
            name,
            side: Side::Right,
            ..default()
        }
    }
    fn left(name: &'static str) -> Self {
        Self {
            name,
            side: Side::Left,
            ..default()
        }
    }
    fn top(name: &'static str) -> Self {
        Self {
            name,
            side: Side::Top,
            ..default()
        }
    }
    fn bottom(name: &'static str) -> Self {
        Self {
            name,
            side: Side::Bottom,
            ..default()
        }
    }
    fn title(mut self) -> Self {
        self.title = true;
        self
    }
    fn close_btn(mut self) -> Self {
        self.close_btn = true;
        self
    }
    fn content(
        mut self,
        content: impl FnOnce(&mut Ui, &mut World) + Send + Sync + 'static,
    ) -> Self {
        self.content = Some(Box::new(content));
        self
    }
    fn child(mut self, child: impl FnOnce(&mut Ui, &mut World) + Send + Sync + 'static) -> Self {
        self.child = Some(Box::new(child));
        self
    }
    fn show(self, ui: &mut Ui, world: &mut World) {
        let content = self.content.unwrap_or(Box::new(|_, _| {}));
        ui.ctx().add_path(&self.name);
        if let Some(child) = self.child {
            child(ui, world);
        }
        let path = ui.ctx().get_path();
        let content = |ui: &mut Ui| {
            if self.title {
                self.name.cstr().label(ui);
            }
            ui.vertical_centered_justified(|ui| {
                content(ui, world);
                if self.close_btn && ui.button_gray("Close").clicked() {
                    ui.ctx().flip_path_open(&path);
                }
            });
        };
        match self.side {
            Side::Right => {
                SidePanel::right(Id::new(&path))
                    .frame(FRAME)
                    .show_separator_line(false)
                    .show_animated_inside(ui, ui.ctx().is_path_open(&path), content);
            }
            Side::Left => {
                SidePanel::left(Id::new(&path))
                    .frame(FRAME)
                    .show_separator_line(false)
                    .show_animated_inside(ui, ui.ctx().is_path_open(&path), content);
            }
            Side::Top => {
                TopBottomPanel::top(Id::new(&path))
                    .frame(FRAME)
                    .show_separator_line(false)
                    .show_animated_inside(ui, ui.ctx().is_path_open(&path), content);
            }
            Side::Bottom => {
                TopBottomPanel::bottom(Id::new(&path))
                    .frame(FRAME)
                    .show_separator_line(false)
                    .show_animated_inside(ui, ui.ctx().is_path_open(&path), content);
            }
        }
        ui.ctx().remove_path();
    }
}

pub struct TilingPlugin;

impl Plugin for TilingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, Self::ui)
            .add_systems(Startup, Self::setup)
            .init_resource::<Data>();
    }
}

impl TilingPlugin {
    fn setup(world: &mut World) {
        let Some(ctx) = &egui_context(world) else {
            return;
        };
        ctx.flip_tile_open("Playback");
    }
    fn ui(world: &mut World) {
        let Some(ctx) = &egui_context(world) else {
            return;
        };
        if just_pressed(KeyCode::Escape, world) {
            ctx.flip_tile_open("Main Menu");
        }
        TopBottomPanel::top("top")
            .frame(Frame::none().inner_margin(Margin::same(4.0)))
            .resizable(false)
            .show_separator_line(false)
            .show(ctx, |ui| {
                ui.with_layout(Layout::right_to_left(egui::Align::Min), |ui| {
                    if let Some(fps) = world
                        .resource::<DiagnosticsStore>()
                        .get(&FrameTimeDiagnosticsPlugin::FPS)
                    {
                        if let Some(fps) = fps.smoothed() {
                            ui.label(format!("fps: {fps:.0}"));
                        }
                    }
                    ui.label(format!("arena-of-ideas {VERSION}"));
                })
            });

        CentralPanel::default()
            .frame(Frame::none())
            .show(ctx, |ui| {
                if matches!(cur_state(world), GameState::Battle) {
                    Tile::bottom("Playback")
                        .content(|ui, world| {})
                        .show(ui, world);
                }
                Tile::right("Main Menu")
                    .title()
                    .close_btn()
                    .content(|ui, _| {
                        let name = "New Game";
                        if ui
                            .button_toggle(name, ui.ctx().is_tile_open(name))
                            .clicked()
                        {
                            ui.ctx().flip_tile_open(name);
                        }
                        let name = "Settings";
                        if ui
                            .button_toggle(name, ui.ctx().is_tile_open(name))
                            .clicked()
                        {
                            ui.ctx().flip_tile_open(name);
                        }
                    })
                    .child(|ui, world| {
                        Tile::right("New Game")
                            .title()
                            .close_btn()
                            .content(|ui, _| {
                                if ui.button("test").clicked() {
                                    debug!("test");
                                }
                            })
                            .show(ui, world);
                        Tile::right("Settings")
                            .title()
                            .close_btn()
                            .content(|ui, world| {
                                if ui.button("setting 1").clicked() {
                                    debug!("Test click");
                                }
                                br(ui);
                                if ui.button("setting 2").clicked() {
                                    debug!("Test click");
                                }
                                br(ui);
                                slider("Test", &mut world.resource_mut::<Data>().t, ui);
                                br(ui);
                            })
                            .show(ui, world);
                    })
                    .show(ui, world);
            });
    }
}

fn slider(name: &str, value: &mut f32, ui: &mut Ui) {
    ui.label(name);
    ui.spacing_mut().slider_width = ui.available_width() - 60.0;
    Slider::new(value, 0.0..=1.0).max_decimals(2).ui(ui);
    ui.reset_style();
}
fn br(ui: &mut Ui) {
    ui.horizontal(|ui| {
        let rect = ui.max_rect();
        let line = egui::Shape::dotted_line(
            &[rect.left_center(), rect.right_center()],
            DARK_GRAY,
            8.0,
            1.5,
        );
        ui.painter().add(line);
    });
}

#[derive(Default, Debug)]
enum Side {
    #[default]
    Right,
    Left,
    Top,
    Bottom,
}

const FRAME: Frame = Frame {
    inner_margin: Margin::same(13.0),
    outer_margin: Margin::same(13.0),
    rounding: Rounding::same(13.0),
    shadow: Shadow::NONE,
    fill: LIGHT_BLACK,
    stroke: Stroke::NONE,
};

const PATH: &str = "tile_path";
trait CtxExt {
    fn is_tile_open(&self, name: &str) -> bool;
    fn is_path_open(&self, path: &str) -> bool;
    fn flip_tile_open(&self, name: &str);
    fn flip_path_open(&self, path: &str);
    fn get_path(&self) -> String;
    fn path_with(&self, name: &str) -> String;
    fn add_path(&self, name: &str);
    fn remove_path(&self);
}

impl CtxExt for egui::Context {
    fn is_tile_open(&self, name: &str) -> bool {
        self.data(|r| r.get_temp(self.path_with(name).into()))
            .unwrap_or_default()
    }
    fn is_path_open(&self, path: &str) -> bool {
        self.data(|r| r.get_temp(Id::new(path))).unwrap_or_default()
    }
    fn flip_tile_open(&self, name: &str) {
        let p = self.path_with(name);
        let v = self.is_path_open(&p);
        self.data_mut(|w| w.insert_temp(p.into(), !v))
    }
    fn flip_path_open(&self, path: &str) {
        let v = self.is_path_open(&path);
        self.data_mut(|w| w.insert_temp(Id::new(path), !v))
    }
    fn get_path(&self) -> String {
        self.data(|r| r.get_temp(Id::new(PATH))).unwrap_or_default()
    }
    fn path_with(&self, name: &str) -> String {
        let p = self.get_path();
        format!("{p}/{name}")
    }
    fn add_path(&self, name: &str) {
        let mut p = self.get_path();
        p.push('/');
        p.push_str(name);
        self.data_mut(|w| w.insert_temp(Id::new(PATH), p));
    }
    fn remove_path(&self) {
        let mut p = self.get_path();
        if let Some(pos) = p.rfind('/') {
            let _ = p.split_off(pos);
            self.data_mut(|w| w.insert_temp(Id::new(PATH), p));
        }
    }
}

trait UiExt {
    fn button_toggle(&mut self, text: impl Into<WidgetText>, value: bool) -> Response;
    fn button_gray(&mut self, text: impl Into<WidgetText>) -> Response;
}

impl UiExt for Ui {
    fn button_toggle(&mut self, text: impl Into<WidgetText>, value: bool) -> Response {
        if value {
            self.style_mut().visuals.widgets.inactive.weak_bg_fill = DARK_GRAY;
            self.style_mut().visuals.widgets.hovered.weak_bg_fill = DARK_GRAY;
        }
        let r = self.button(text);
        self.reset_style();
        r
    }

    fn button_gray(&mut self, text: impl Into<WidgetText>) -> Response {
        self.style_mut().visuals.widgets.inactive.fg_stroke.color = LIGHT_GRAY;
        self.style_mut().visuals.widgets.hovered.fg_stroke.color = LIGHT_GRAY;
        let r = self.button(text);
        self.reset_style();
        r
    }
}
