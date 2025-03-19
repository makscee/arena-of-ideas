use std::ops::Deref;

use super::*;

pub struct Button {
    name: String,
    title: Option<String>,
    icon: Option<Icon>,
    min_width: f32,
    enabled: bool,
    active: bool,
    original_style: Option<egui::Style>,
}

impl Default for Button {
    fn default() -> Self {
        Self {
            name: Default::default(),
            title: Default::default(),
            enabled: true,
            active: false,
            icon: None,
            min_width: 0.0,
            original_style: None,
        }
    }
}

impl Button {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }
    fn save_style(&mut self, ui: &mut Ui) {
        if self.original_style.is_none() {
            self.original_style = Some(ui.style().deref().clone());
        }
    }
    pub fn color(self, color: Color32, ui: &mut Ui) -> Self {
        let style = ui.style_mut();
        style.visuals.widgets.inactive.fg_stroke.color = color;
        self
    }
    pub fn gray(self, ui: &mut Ui) -> Self {
        self.color(tokens_global().subtle_background(), ui)
    }
    pub fn red(mut self, ui: &mut Ui) -> Self {
        self.save_style(ui);
        colorix().style_error(ui);
        self
    }
    pub fn bg(self, ui: &mut Ui) -> Self {
        let style = ui.style_mut();
        style.visuals.widgets.inactive.weak_bg_fill = tokens_global().subtle_background();
        style.visuals.widgets.hovered.weak_bg_fill = tokens_global().solid_backgrounds();
        self
    }
    pub fn set_bg(self, value: bool, ui: &mut Ui) -> Self {
        if value {
            self.bg(ui)
        } else {
            self
        }
    }
    pub fn title(mut self, text: String) -> Self {
        self.title = Some(text);
        self
    }
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }
    pub fn min_width(mut self, width: f32) -> Self {
        self.min_width = width;
        self
    }
    pub fn enabled(mut self, value: bool) -> Self {
        self.enabled = value;
        self
    }
    pub fn active(mut self, value: bool) -> Self {
        self.active = value;
        self
    }
    pub fn ui(self, ui: &mut Ui) -> Response {
        if let Some(title) = self.title {
            title.label(ui);
        }
        let mut job = self.name.job(1.0, ui.style());
        let mut replace_color = |c: Color32| {
            job.sections[0].format.color = c;
            job.sections[0].byte_range = 0..job.text.len();
            job.sections.truncate(1);
        };
        // if !self.enabled {
        //     style.visuals.widgets.noninteractive.bg_stroke.color = TRANSPARENT;
        //     style.visuals.widgets.noninteractive.fg_stroke.color = tokens_global().low_contrast_text();
        //     replace_color(tokens_global().low_contrast_text());
        // } else if self.active {
        //     style.visuals.widgets.inactive.fg_stroke.color = YELLOW;
        //     style.visuals.widgets.hovered.fg_stroke.color = YELLOW;
        //     style.visuals.widgets.inactive.bg_stroke.color = YELLOW;
        //     style.visuals.widgets.hovered.bg_stroke.color = YELLOW;
        //     replace_color(YELLOW);
        // }
        let sense = if self.enabled {
            Sense::click()
        } else {
            Sense::hover()
        };
        let r = if let Some(icon) = self.icon {
            egui::ImageButton::new(icon.image()).sense(sense).ui(ui)
        } else {
            egui::Button::new(WidgetText::LayoutJob(job))
                .wrap_mode(egui::TextWrapMode::Extend)
                .sense(sense)
                .min_size(egui::vec2(self.min_width, 0.0))
                .ui(ui)
        };
        if r.clicked() {
            // AudioPlugin::queue_sound(SoundEffect::Click);
        }
        if let Some(style) = self.original_style {
            ui.set_style(style);
        }
        r
    }
}
