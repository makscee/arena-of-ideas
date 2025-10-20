use std::{ops::Deref, sync::Arc};

use super::*;

pub struct Button {
    name: String,
    title: Option<String>,
    icon: Option<Icon>,
    min_width: f32,
    enabled: bool,
    original_style: Option<egui::Style>,
}

impl Default for Button {
    fn default() -> Self {
        Self {
            name: Default::default(),
            title: Default::default(),
            enabled: true,
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
        let style = ui.style_mut();
        style.visuals.widgets.inactive.weak_bg_fill = subtle_background();
        self
    }
    pub fn red(self, ui: &mut Ui) -> Self {
        let stroke = Stroke::new(2.0, RED);
        ui.style_mut().visuals.widgets.inactive.bg_stroke = stroke;
        ui.style_mut().visuals.widgets.hovered.bg_stroke = stroke;
        self
    }
    pub fn danger(self, ui: &mut Ui) -> Self {
        let style = ui.style_mut();
        style.visuals.widgets.inactive.weak_bg_fill = subtle_background();
        style.visuals.widgets.hovered.weak_bg_fill = solid_backgrounds();
        self
    }
    pub fn set_bg(self, value: bool, ui: &mut Ui) -> Self {
        if value { self.danger(ui) } else { self }
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
    pub fn active(mut self, value: bool, ui: &mut Ui) -> Self {
        if value {
            self.save_style(ui);
            let stroke = Stroke::new(1.0, YELLOW);
            ui.style_mut().visuals.widgets.inactive.fg_stroke = stroke;
            ui.style_mut().visuals.widgets.hovered.fg_stroke = stroke;
            // ui.style_mut().visuals.widgets.inactive.bg_stroke = stroke;
            // ui.style_mut().visuals.widgets.hovered.bg_stroke = stroke;
        }
        self
    }
    pub fn ui(self, ui: &mut Ui) -> Response {
        if let Some(title) = self.title {
            title.label(ui);
        }
        let job = self.name.job(1.0, ui.style());
        // let mut replace_color = |c: Color32| {
        //     job.sections[0].format.color = c;
        //     job.sections[0].byte_range = 0..job.text.len();
        //     job.sections.truncate(1);
        // };
        // Color styling handled by colorix
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
            egui::Button::image(icon.image()).sense(sense).ui(ui)
        } else {
            egui::Button::new(WidgetText::LayoutJob(Arc::new(job)))
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
