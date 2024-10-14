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
    pub fn ui_string(self, value: &mut String, ui: &mut Ui) {
        if ui.available_width() < 10.0 {
            return;
        }
        self.name.cstr().label(ui);
        ui.style_mut().visuals.widgets.inactive.bg_stroke = STROKE_DARK;
        TextEdit::singleline(value)
            .password(self.password)
            .desired_width(100.0)
            .ui(ui);
        ui.reset_style();
    }
}
