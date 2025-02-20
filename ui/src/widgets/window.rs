use bevy::log::debug;

use super::*;

pub struct WindowPlugin;

impl Plugin for WindowPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WindowResource>();
    }
}

pub struct Window {
    id: String,
    order: Order,
    no_frame: bool,
    transparent: bool,
    expand: bool,
    center_anchor: bool,
    default_width: f32,
    default_height: f32,
    content: Box<dyn FnMut(&mut Ui, &mut World) + Send + Sync>,
}

enum WindowResponse {
    None,
    Tile(Side),
    Close,
}

#[derive(Resource, Default)]
struct WindowResource {
    windows: HashMap<String, Window>,
}
#[derive(Resource)]
struct CloseCurrentWindow;
fn rm(world: &mut World) -> Mut<WindowResource> {
    world.resource_mut::<WindowResource>()
}
fn r(world: &World) -> &WindowResource {
    world.resource::<WindowResource>()
}

impl Window {
    #[must_use]
    pub fn new(
        id: impl ToString,
        content: impl FnMut(&mut Ui, &mut World) + Send + Sync + 'static,
    ) -> Self {
        Self {
            id: id.to_string(),
            content: Box::new(content),
            order: Order::Middle,
            no_frame: false,
            transparent: false,
            expand: false,
            center_anchor: false,
            default_width: 150.0,
            default_height: 100.0,
        }
    }
    #[must_use]
    pub fn order(mut self, order: Order) -> Self {
        self.order = order;
        self
    }
    #[must_use]
    pub fn no_frame(mut self) -> Self {
        self.no_frame = true;
        self
    }
    #[must_use]
    pub fn transparent(mut self) -> Self {
        self.transparent = true;
        self
    }
    #[must_use]
    pub fn center_anchor(mut self) -> Self {
        self.center_anchor = true;
        self
    }
    #[must_use]
    pub fn default_width(mut self, value: f32) -> Self {
        self.default_width = value;
        self
    }
    #[must_use]
    pub fn default_height(mut self, value: f32) -> Self {
        self.default_height = value;
        self
    }
    pub fn push(self, world: &mut World) {
        rm(world).windows.insert(self.id.clone(), self);
    }
    fn show(&mut self, ctx: &egui::Context, world: &mut World) -> WindowResponse {
        const FRAME: Frame = Frame {
            inner_margin: MARGIN,
            outer_margin: Margin::ZERO,
            corner_radius: ROUNDING,
            shadow: SHADOW,
            fill: EMPTINESS,
            stroke: STROKE_DARK,
        };
        let mut r = WindowResponse::None;
        let mut w = egui::Window::new(&self.id)
            .default_width(self.default_width)
            .default_height(self.default_height)
            .title_bar(false)
            .frame(if self.no_frame { Frame::none() } else { FRAME })
            .scroll([self.expand, self.expand])
            .movable(!self.expand)
            .order(self.order);
        if self.expand {
            w = w.fixed_rect(ctx.screen_rect().shrink(13.0));
        }
        if self.center_anchor {
            w = w.anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO);
        }
        w.show(ctx, |ui| {
            ui.expand_to_include_rect(ui.available_rect_before_wrap());
            if self.no_frame {
                (self.content)(ui, world);
                return;
            }
            let rect = CollapsingHeader::new(&self.id)
                .default_open(true)
                .show_unindented(ui, |ui| {
                    (self.content)(ui, world);
                })
                .header_response
                .rect;
            let rect = {
                let rect = rect.with_min_x(rect.max.x);
                let ui = &mut ui.child_ui(rect, Layout::left_to_right(Align::Max), None);
                if "o"
                    .cstr_c(if self.expand { YELLOW } else { VISIBLE_LIGHT })
                    .button(ui)
                    .clicked()
                {
                    self.expand = !self.expand
                }
                if "<".cstr().button(ui).clicked() {
                    r = WindowResponse::Tile(Side::Left);
                }
                if ">".cstr().button(ui).clicked() {
                    r = WindowResponse::Tile(Side::Right);
                }
                if "x".cstr().button(ui).clicked() {
                    r = WindowResponse::Close;
                }
                ui.min_rect()
            };
            ui.expand_to_include_rect(rect);
        });
        r
    }
}

impl WindowPlugin {
    pub fn show_all(ctx: &egui::Context, world: &mut World) {
        let mut windows = mem::take(&mut rm(world).windows);
        let mut close = None;
        let mut tile = None;
        for (id, window) in windows.iter_mut() {
            match window.show(ctx, world) {
                WindowResponse::None => {}
                WindowResponse::Close => close = Some(id.clone()),
                WindowResponse::Tile(side) => tile = Some((side, id.clone())),
            }
            if world.remove_resource::<CloseCurrentWindow>().is_some() {
                debug!("close window");
                close = Some(id.clone());
            }
        }
        if let Some(close) = close {
            windows.remove(&close);
        }
        if let Some((side, tile)) = tile {
            let w = windows.remove(&tile).unwrap();
            Tile::new(side, w.content).push(world);
        }
        let mut r = rm(world);
        windows.extend(r.windows.drain());
        r.windows = windows;
    }
    pub fn is_open(key: &str, world: &World) -> bool {
        r(world).windows.contains_key(key)
    }
    pub fn close(key: &str, world: &mut World) {
        rm(world).windows.remove(key);
    }
    pub fn close_current(world: &mut World) {
        world.insert_resource(CloseCurrentWindow);
    }
}
