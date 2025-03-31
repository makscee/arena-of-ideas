use ecolor::Rgba;

use super::*;

pub struct Input {
    name: String,
    id: Option<Id>,
    password: bool,
    char_limit: usize,
    desired_width: f32,
    override_color: Option<Color32>,
}

impl Input {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: None,
            name: name.into(),
            password: false,
            char_limit: 0,
            desired_width: f32::INFINITY,
            override_color: None,
        }
    }
    pub fn id(mut self, value: impl Into<Id>) -> Self {
        self.id = Some(value.into());
        self
    }
    pub fn password(mut self) -> Self {
        self.password = true;
        self
    }
    pub fn char_limit(mut self, limit: usize) -> Self {
        self.char_limit = limit;
        self
    }
    pub fn desired_width(mut self, value: f32) -> Self {
        self.desired_width = value;
        self
    }
    pub fn color(self, value: Color32) -> Self {
        self.color_opt(Some(value))
    }
    pub fn color_opt(mut self, value: Option<Color32>) -> Self {
        self.override_color = value;
        self
    }
    pub fn ui_string(self, value: &mut String, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            self.name.label(ui);
            let mut te = TextEdit::singleline(value)
                .desired_width(self.desired_width)
                .password(self.password);
            if let Some(color) = self.override_color {
                te = te.text_color(color);
                if Rgba::from(color).intensity() < 0.05 {
                    te = te.background_color(tokens_global().high_contrast_text());
                }
            }
            if let Some(id) = self.id {
                te = te.id(id);
            }
            if self.char_limit > 0 {
                te = te.char_limit(self.char_limit);
            }
            te.ui(ui)
        })
        .inner
    }
}
