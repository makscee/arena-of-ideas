use super::*;

#[derive(Clone)]
pub struct Confirmation {
    text: Cstr,
    accept: fn(&mut World),
    accept_name: Option<String>,
    decline: fn(&mut World),
    decline_name: Option<String>,
    decline_hide: bool,
    content: Option<fn(&mut Ui, &mut World)>,
}

fn current_id() -> Id {
    static TILES_DATA: OnceCell<Id> = OnceCell::new();
    *TILES_DATA.get_or_init(|| Id::new("tiles_data"))
}

impl Confirmation {
    #[must_use]
    pub fn new(text: Cstr, accept: fn(&mut World)) -> Self {
        Self {
            text,
            accept,
            decline: |_| {},
            accept_name: None,
            decline_name: None,
            content: None,
            decline_hide: false,
        }
    }
    #[must_use]
    pub fn decline(mut self, action: fn(&mut World)) -> Self {
        self.decline = action;
        self
    }
    #[must_use]
    pub fn content(mut self, content: fn(&mut Ui, &mut World)) -> Self {
        self.content = Some(content);
        self
    }
    #[must_use]
    pub fn accept_name(mut self, name: String) -> Self {
        self.accept_name = Some(name);
        self
    }
    #[must_use]
    pub fn decline_name(mut self, name: String) -> Self {
        self.decline_name = Some(name);
        self
    }
    #[must_use]
    pub fn popup(mut self) -> Self {
        self.decline_hide = true;
        self.accept_name = Some("Close".into());
        self
    }
    fn ui(self, ctx: &egui::Context, world: &mut World) {
        popup("Confirmation window", ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                self.text.as_label(ui).wrap().ui(ui);
                if let Some(content) = self.content {
                    br(ui);
                    (content)(ui, world);
                }
            });
            space(ui);
            ui.columns(2, |ui| {
                ui[0].vertical_centered_justified(|ui| {
                    if !self.decline_hide
                        && Button::click(self.decline_name.unwrap_or("Decline".into()))
                            .red(ui)
                            .ui(ui)
                            .clicked()
                    {
                        Self::clear(ui.ctx());
                        (self.decline)(world);
                    }
                });
                ui[1].vertical_centered_justified(|ui| {
                    if Button::click(self.accept_name.unwrap_or("Accept".into()))
                        .ui(ui)
                        .clicked()
                    {
                        Self::clear(ui.ctx());
                        (self.accept)(world);
                    }
                });
            })
        })
    }

    pub fn push(self, ctx: &egui::Context) {
        ctx.data_mut(|w| w.insert_temp(current_id(), self));
    }
    pub fn clear(ctx: &egui::Context) {
        ctx.data_mut(|w| w.remove_by_type::<Confirmation>());
    }
    fn data(ctx: &egui::Context) -> Option<Self> {
        ctx.data(|r| r.get_temp::<Self>(current_id()))
    }
    pub fn show_current(ctx: &egui::Context, world: &mut World) {
        let Some(c) = Self::data(ctx) else {
            return;
        };
        c.ui(ctx, world);
    }
    pub fn has_active(ctx: &egui::Context) -> bool {
        Self::data(ctx).is_some()
    }
}
