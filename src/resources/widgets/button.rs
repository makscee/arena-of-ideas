use egui::Sense;

use super::*;

pub struct Button {
    name: String,
    show_name: Option<Cstr>,
    title: Option<String>,
    enabled: bool,
}

impl Default for Button {
    fn default() -> Self {
        Self {
            name: default(),
            title: default(),
            enabled: true,
            show_name: None,
        }
    }
}

impl Button {
    pub fn click(name: String) -> Self {
        Self { name, ..default() }
    }
    pub fn cstr(mut self, name: Cstr) -> Self {
        self.show_name = Some(name);
        self
    }
    pub fn gray(self, ui: &mut Ui) -> Self {
        let style = ui.style_mut();
        style.visuals.widgets.inactive.fg_stroke.color = VISIBLE_DARK;
        style.visuals.widgets.hovered.fg_stroke.color = VISIBLE_DARK;
        self
    }
    pub fn bg(self, ui: &mut Ui) -> Self {
        let style = ui.style_mut();
        style.visuals.widgets.inactive.weak_bg_fill = BG_LIGHT;
        style.visuals.widgets.hovered.weak_bg_fill = BG_LIGHT;
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
    pub fn enabled(mut self, value: bool) -> Self {
        self.enabled = value;
        self
    }
    pub fn enable_ui<T: Default>(self, data: &mut Option<T>, ui: &mut Ui) -> Response {
        self.enable_ui_with(data, default, ui)
    }
    pub fn enable_ui_with<T>(
        mut self,
        data: &mut Option<T>,
        init: impl FnOnce() -> T,
        ui: &mut Ui,
    ) -> Response {
        self = self.set_bg(data.is_some(), ui);
        let r = self.ui(ui);
        if r.clicked() {
            if data.is_some() {
                *data = None;
            } else {
                *data = Some(init());
            }
        }
        r
    }
    pub fn ui(self, ui: &mut Ui) -> Response {
        if let Some(title) = self.title {
            title.cstr().label(ui);
        }
        if !self.enabled {
            let style = ui.style_mut();
            style.visuals.widgets.noninteractive.bg_stroke.color = TRANSPARENT;
            style.visuals.widgets.noninteractive.fg_stroke.color = VISIBLE_DARK;
        }
        let r = if let Some(show) = self.show_name {
            egui::Button::new(show.widget(1.0, ui))
        } else {
            egui::Button::new(self.name)
        }
        .wrap(false)
        .sense(if self.enabled {
            Sense::click()
        } else {
            Sense::hover()
        })
        .ui(ui);
        ui.reset_style();
        r
    }
}
