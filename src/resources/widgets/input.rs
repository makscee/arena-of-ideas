use egui::TextEdit;

use super::*;

pub struct Input {
    name: &'static str,
    password: bool,
}

impl Input {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            password: false,
        }
    }
    pub fn password(mut self) -> Self {
        self.password = true;
        self
    }
    pub fn ui(self, value: &mut String, ui: &mut Ui) {
        ui.columns(2, |ui| {
            self.name.cstr().push(":".cstr()).label(&mut ui[0]);
            TextEdit::singleline(value)
                .password(self.password)
                .ui(&mut ui[1]);
        });
    }
}
