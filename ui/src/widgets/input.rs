use super::*;

pub struct Input {
    name: String,
    password: bool,
    char_limit: usize,
}

impl Input {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
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
    pub fn ui_string(self, value: &mut String, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            self.name.label(ui);
            ui.style_mut().visuals.widgets.inactive.bg_stroke = STROKE_BG_DARK;
            let mut te = TextEdit::singleline(value)
                .desired_width(f32::INFINITY)
                .password(self.password);
            if self.char_limit > 0 {
                te = te.char_limit(self.char_limit);
            }
            let r = te.ui(ui);
            ui.reset_style();
            r
        })
        .inner
    }
}
