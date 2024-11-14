use super::*;

pub struct ConfirmationPlugin;
impl Plugin for ConfirmationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ConfirmationResource>();
    }
}

pub struct Confirmation {
    text: Cstr,
    accept: Option<Box<dyn Fn(&mut World) + Send + Sync>>,
    accept_name: String,
    cancel: Option<Box<dyn Fn(&mut World) + Send + Sync>>,
    cancel_name: String,
    content: Option<Box<dyn Fn(&mut Ui, &mut World) + Send + Sync>>,
}

#[derive(Resource, Default)]
struct ConfirmationResource {
    stack: Vec<Confirmation>,
    new: Vec<Confirmation>,
    close_requested: bool,
}
fn rm(world: &mut World) -> Mut<ConfirmationResource> {
    world.resource_mut::<ConfirmationResource>()
}

impl Confirmation {
    #[must_use]
    pub fn new(text: Cstr) -> Self {
        Self {
            text,
            accept: None,
            accept_name: "Accept".into(),
            cancel: None,
            cancel_name: "Cancel".into(),
            content: None,
        }
    }
    #[must_use]
    pub fn accept(mut self, action: impl Fn(&mut World) + Send + Sync + 'static) -> Self {
        self.accept = Some(Box::new(action));
        self
    }
    #[must_use]
    pub fn cancel(mut self, action: impl Fn(&mut World) + Send + Sync + 'static) -> Self {
        self.cancel = Some(Box::new(action));
        self
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
        content: impl Fn(&mut Ui, &mut World) + Send + Sync + 'static,
    ) -> Self {
        self.content = Some(Box::new(content));
        self
    }
    fn ui(&self, ctx: &egui::Context, world: &mut World) {
        popup("Confirmation window", ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                self.text.as_label(ui).wrap().ui(ui);
            });
            ui.vertical(|ui| {
                if let Some(content) = &self.content {
                    (content)(ui, world);
                }
            });
            space(ui);
            ui.columns(2, |ui| {
                ui[0].vertical_centered_justified(|ui| {
                    if let Some(cancel) = &self.cancel {
                        if Button::click(&self.cancel_name).red(ui).ui(ui).clicked() {
                            Self::close_current(world);
                            cancel(world);
                        }
                    }
                });
                ui[1].vertical_centered_justified(|ui| {
                    if let Some(accept) = &self.accept {
                        if Button::click(&self.accept_name).ui(ui).clicked() {
                            Self::close_current(world);
                            accept(world);
                        }
                    }
                });
            })
        })
    }

    pub fn push(self, world: &mut World) {
        rm(world).new.push(self);
    }
    pub fn close_current(world: &mut World) {
        rm(world).close_requested = true;
    }
    pub fn show_current(ctx: &egui::Context, world: &mut World) {
        if let Some(c) = rm(world).stack.pop() {
            c.ui(ctx, world);
            let esc = c.cancel.is_some() && just_pressed(KeyCode::Escape, world);
            let mut r = rm(world);
            if r.close_requested || esc {
                r.close_requested = false;
            } else {
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
