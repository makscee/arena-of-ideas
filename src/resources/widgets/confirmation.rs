use super::*;

#[derive(Clone)]
pub struct Confirmation {
    text: Cstr,
    accept: Option<fn(&mut World)>,
    accept_name: String,
    cancel: Option<fn(&mut World)>,
    cancel_name: String,
    content: Option<fn(&mut Ui, &mut World)>,
}

fn id() -> Id {
    static ID: OnceCell<Id> = OnceCell::new();
    *ID.get_or_init(|| Id::new("confirmation_id"))
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
    pub fn accept(mut self, action: fn(&mut World)) -> Self {
        self.accept = Some(action);
        self
    }
    #[must_use]
    pub fn cancel(mut self, action: fn(&mut World)) -> Self {
        self.cancel = Some(action);
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
    pub fn content(mut self, content: fn(&mut Ui, &mut World)) -> Self {
        self.content = Some(content);
        self
    }
    #[must_use]
    pub fn popup(mut self) -> Self {
        self.accept_name = "Close".into();
        self.accept = Some(|_| {});
        self
    }
    fn ui(self, ctx: &egui::Context, world: &mut World) {
        popup("Confirmation window", ctx, |ui| {
            ui.vertical(|ui| {
                self.text.as_label(ui).wrap().ui(ui);
                if let Some(content) = self.content {
                    br(ui);
                    (content)(ui, world);
                }
            });
            space(ui);
            ui.columns(2, |ui| {
                ui[0].vertical_centered_justified(|ui| {
                    if let Some(cancel) = self.cancel {
                        if Button::click(self.cancel_name).red(ui).ui(ui).clicked() {
                            Self::close_current(ui.ctx());
                            cancel(world);
                        }
                    }
                });
                ui[1].vertical_centered_justified(|ui| {
                    if let Some(accept) = self.accept {
                        if Button::click(self.accept_name).ui(ui).clicked() {
                            Self::close_current(ui.ctx());
                            accept(world);
                        }
                    }
                });
            })
        })
    }

    pub fn push(self, ctx: &egui::Context) {
        ctx.data_mut(|w| w.get_temp_mut_or_default::<Vec<Self>>(id()).push(self));
    }
    pub fn close_current(ctx: &egui::Context) {
        ctx.data_mut(|w| w.get_temp_mut_or_default::<Vec<Self>>(id()).pop());
    }
    fn data(ctx: &egui::Context) -> Vec<Self> {
        ctx.data(|r| r.get_temp::<Vec<Self>>(id()).unwrap_or_default())
    }
    pub fn show_current(ctx: &egui::Context, world: &mut World) {
        if let Some(c) = Self::data(ctx).pop() {
            c.ui(ctx, world);
        }
    }
    pub fn has_active(ctx: &egui::Context) -> bool {
        !Self::data(ctx).is_empty()
    }
}
