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
    no_frame: bool,
    content: Box<dyn Fn(&mut Ui, &mut World) + Send + Sync>,
}

#[derive(Resource, Default)]
struct WindowResource {
    windows: HashMap<String, Window>,
}
fn rm(world: &mut World) -> Mut<WindowResource> {
    world.resource_mut::<WindowResource>()
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
            no_frame: false,
        }
    }
    pub fn no_frame(mut self) -> Self {
        self.no_frame = true;
        self
    }
    pub fn push(self, world: &mut World) {
        rm(world).windows.insert(self.id.clone(), self);
    }
    fn show(&self, ctx: &egui::Context, world: &mut World) {
        let mut w = egui::Window::new(&self.id).title_bar(false);
        if self.no_frame {
            w = w.frame(Frame::none());
        };
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
}
