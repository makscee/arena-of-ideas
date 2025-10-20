use bevy_egui::egui::UiKind;

use super::*;

pub struct ContextMenu {
    response: Response,
    buttons: Vec<(
        String,
        Box<dyn Fn(&mut Ui, &mut World) + 'static + Send + Sync>,
    )>,
}

impl ContextMenu {
    pub fn new(response: Response) -> Self {
        Self {
            response,
            buttons: default(),
        }
    }
    pub fn add(
        mut self,
        name: impl ToString,
        f: impl Fn(&mut Ui, &mut World) + 'static + Send + Sync,
    ) -> Self {
        self.buttons.push((name.to_string(), Box::new(f)));
        self
    }
    pub fn ui(self, ui: &mut Ui, world: &mut World) {
        let bar_id = ui.id();
        let mut bar_state = egui::menu::BarState::load(ui.ctx(), bar_id);
        bar_state.bar_menu(&self.response, |ui| {
            for (name, action) in self.buttons {
                if ui.button(name).clicked() {
                    action(ui, world);
                    ui.close_kind(UiKind::Menu);
                }
            }
            if "[tl Close]".cstr().button(ui).clicked() {
                ui.close_kind(UiKind::Menu);
            }
        });
        bar_state.store(ui.ctx(), bar_id);
    }
}
