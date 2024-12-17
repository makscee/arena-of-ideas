use bevy::utils::hashbrown::HashMap;

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
    content: Box<dyn Fn(&mut Ui, &mut World) + Send + Sync>,
}

#[derive(Resource, Default)]
struct WindowResource {
    windows: HashMap<String, Window>,
}
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
        content: impl Fn(&mut Ui, &mut World) + Send + Sync + 'static,
    ) -> Self {
        Self {
            id: id.to_string(),
            content: Box::new(content),
            order: Order::Middle,
            no_frame: false,
            transparent: false,
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
    pub fn push(self, world: &mut World) {
        rm(world).windows.insert(self.id.clone(), self);
    }
    pub fn show(&self, ctx: &egui::Context, world: &mut World) {
        let mut w = egui::Window::new(&self.id)
            .title_bar(false)
            .order(self.order);
        if self.no_frame {
            w = w.frame(Frame::none());
        }
        w.show(ctx, |ui| {
            (self.content)(ui, world);
        });
    }
}

impl WindowPlugin {
    pub fn show_all(ctx: &egui::Context, world: &mut World) {
        let mut windows = mem::take(&mut rm(world).windows);
        for window in windows.values() {
            window.show(ctx, world);
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
}
