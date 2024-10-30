use egui::TextEdit;

use super::*;

pub struct Input {
    name: &'static str,
    password: bool,
    char_limit: usize,
}

impl Input {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            password: false,
            char_limit: 0,
        }
    }
    pub fn password(mut self) -> Self {
        self.password = true;
        self
    }
    pub fn char_limit(mut self, limit: usize) -> Self {
        self.char_limit = limit;
        self
    }
    pub fn ui_string(self, value: &mut String, ui: &mut Ui) {
        if ui.available_width() < 10.0 {
            return;
        }
        self.name.cstr().label(ui);
        ui.style_mut().visuals.widgets.inactive.bg_stroke = STROKE_DARK;
        let mut te = TextEdit::singleline(value)
            .password(self.password)
            .desired_width(100.0);
        if self.char_limit > 0 {
            te = te.char_limit(self.char_limit);
        }
        te.ui(ui);
        ui.reset_style();
    }
}
