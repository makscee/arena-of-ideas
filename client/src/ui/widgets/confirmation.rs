use super::*;

pub struct ConfirmationPlugin;
impl Plugin for ConfirmationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ConfirmationResource>();
    }
}

pub struct Confirmation {
    text: Cstr,
    accept_name: String,
    cancel_name: String,
    content: Option<Box<dyn FnMut(&mut Ui, &mut World, Option<bool>) + Send + Sync>>,
    fullscreen: bool,
}

#[derive(Resource, Default)]
struct ConfirmationResource {
    stack: Vec<Confirmation>,
    new: Vec<Confirmation>,
    close_requested: Option<bool>,
}
fn rm<'a>(world: &'a mut World) -> Mut<'a, ConfirmationResource> {
    world.resource_mut::<ConfirmationResource>()
}

impl Confirmation {
    #[must_use]
    pub fn new(text: &str) -> Self {
        Self {
            text: text.cstr_s(CstrStyle::Heading2),
            accept_name: "Accept".into(),
            cancel_name: "Cancel".into(),
            content: None,
            fullscreen: false,
        }
    }
    #[must_use]
    pub fn accept_name(mut self, name: impl Into<String>) -> Self {
        self.accept_name = name.into();
        self
    }
    #[must_use]
    pub fn cancel_name(mut self, name: impl Into<String>) -> Self {
        self.cancel_name = name.into();
        self
    }
    #[must_use]
    pub fn content(
        mut self,
        content: impl FnMut(&mut Ui, &mut World, Option<bool>) + Send + Sync + 'static,
    ) -> Self {
        self.content = Some(Box::new(content));
        self
    }
    #[must_use]
    pub fn fullscreen(mut self) -> Self {
        self.fullscreen = true;
        self
    }
    fn ui(&mut self, ctx: &egui::Context, world: &mut World) {
        popup("Confirmation window", self.fullscreen, ctx, |ui| {
            ui.vertical_centered(|ui| {
                self.text.as_label(ui.style()).wrap().ui(ui);
            });
            if let Some(content) = &mut self.content {
                let r = rm(world).close_requested;
                let rect = ui.ctx().content_rect();
                ScrollArea::both()
                    .min_scrolled_width(700.0)
                    .max_width(rect.width() - 30.0)
                    .max_height(rect.height() - 100.0)
                    .show(ui, |ui| {
                        (content)(ui, world, r);
                    });
            }
            ui.columns(2, |ui| {
                ui[0].vertical_centered_justified(|ui| {
                    if Button::new(&self.cancel_name).ui(ui).clicked() || ui.key_down(Key::Escape) {
                        Self::close_current(false);
                    }
                });
                ui[1].vertical_centered_justified(|ui| {
                    if Button::new(&self.accept_name).ui(ui).clicked() {
                        Self::close_current(true);
                    }
                });
            });
        })
    }

    pub fn push(self, world: &mut World) {
        rm(world).new.push(self);
    }
    pub fn push_op(self) {
        op(move |world| {
            self.push(world);
        })
    }
    pub fn close_current(accepted: bool) {
        op(move |world| {
            rm(world).close_requested = Some(accepted);
        });
    }
    pub fn pop(world: &mut World) -> Option<Self> {
        rm(world).stack.pop()
    }
    pub fn show_current(ctx: &egui::Context, world: &mut World) {
        if let Some(mut c) = rm(world).stack.pop() {
            c.ui(ctx, world);
            let mut r = rm(world);
            if !r.close_requested.take().is_some() {
                r.stack.push(c);
            }
        }
        let mut r = rm(world);
        let new = mem::take(&mut r.new);
        r.stack.extend(new);
    }
    pub fn has_active(world: &mut World) -> bool {
        !rm(world).stack.is_empty()
    }
}
