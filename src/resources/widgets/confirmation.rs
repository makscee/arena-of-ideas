use super::*;

#[derive(Clone)]
pub struct Confirmation {
    text: Cstr,
    accept: fn(&mut World),
    accept_name: Option<String>,
    decline: fn(&mut World),
    decline_name: Option<String>,
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
        }
    }
    pub fn decline(mut self, action: fn(&mut World)) -> Self {
        self.decline = action;
        self
    }
    pub fn accept_name(mut self, name: String) -> Self {
        self.accept_name = Some(name);
        self
    }
    pub fn decline_name(mut self, name: String) -> Self {
        self.decline_name = Some(name);
        self
    }
    pub fn ui(self, ctx: &egui::Context, world: &mut World) {
        popup("Confirmation window", ctx, |ui| {
            space(ui);
            ui.vertical_centered_justified(|ui| {
                self.text.as_label(ui).wrap().ui(ui);
            });
            space(ui);
            ui.columns(2, |ui| {
                ui[0].vertical_centered_justified(|ui| {
                    if Button::click(self.decline_name.unwrap_or("Decline".into()))
                        .red(ui)
                        .ui(ui)
                        .clicked()
                    {
                        (self.decline)(world);
                        Self::clear(ui.ctx());
                    }
                });
                ui[1].vertical_centered_justified(|ui| {
                    if Button::click(self.accept_name.unwrap_or("Accept".into()))
                        .ui(ui)
                        .clicked()
                    {
                        (self.accept)(world);
                        Self::clear(ui.ctx());
                    }
                });
            })
        })
    }

    pub fn add(self, ctx: &egui::Context) {
        ctx.data_mut(|w| w.insert_temp(current_id(), self));
    }
    pub fn clear(ctx: &egui::Context) {
        ctx.data_mut(|w| w.remove_by_type::<Confirmation>());
    }
    pub fn show_current(ctx: &egui::Context, world: &mut World) {
        let Some(c) = ctx.data(|r| r.get_temp::<Confirmation>(current_id())) else {
            return;
        };
        c.ui(ctx, world);
    }
}
