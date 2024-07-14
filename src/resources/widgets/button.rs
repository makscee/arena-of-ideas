use egui::Sense;

use super::*;

pub struct Button {
    name: String,
    variant: ButtonVariant,
    title: Option<String>,
    enabled: bool,
}

#[derive(Default)]
enum ButtonVariant {
    #[default]
    Click,
    ToggleChild,
}

impl Default for Button {
    fn default() -> Self {
        Self {
            name: default(),
            variant: default(),
            title: default(),
            enabled: true,
        }
    }
}

impl Button {
    pub fn click(name: String) -> Self {
        Self { name, ..default() }
    }
    pub fn toggle_child(name: String) -> Self {
        Self {
            name,
            variant: ButtonVariant::ToggleChild,
            ..default()
        }
    }
    pub fn gray(self, ui: &mut Ui) -> Self {
        let style = ui.style_mut();
        style.visuals.widgets.inactive.fg_stroke.color = VISIBLE_LIGHT;
        style.visuals.widgets.hovered.fg_stroke.color = VISIBLE_LIGHT;
        self
    }
    pub fn bg(self, ui: &mut Ui) -> Self {
        let style = ui.style_mut();
        style.visuals.widgets.inactive.weak_bg_fill = VISIBLE_DARK;
        style.visuals.widgets.hovered.weak_bg_fill = VISIBLE_DARK;
        self
    }
    pub fn title(mut self, text: String) -> Self {
        self.title = Some(text);
        self
    }
    pub fn enabled(mut self, value: bool) -> Self {
        self.enabled = value;
        self
    }
    pub fn ui(mut self, ui: &mut Ui) -> Response {
        ui.ctx().add_path(&self.name);
        let path = ui.ctx().path();

        match self.variant {
            ButtonVariant::Click => {}
            ButtonVariant::ToggleChild => {
                if ui.ctx().is_path_enabled(&path) {
                    self = self.bg(ui);
                }
            }
        }
        if let Some(title) = self.title {
            title.cstr().label(ui);
        }

        if !self.enabled {
            let style = ui.style_mut();
            style.visuals.widgets.noninteractive.bg_stroke.color = TRANSPARENT;
            style.visuals.widgets.noninteractive.fg_stroke.color = VISIBLE_DARK;
        }
        let r = egui::Button::new(self.name)
            .wrap(false)
            .sense(if self.enabled {
                Sense::click()
            } else {
                Sense::hover()
            })
            .ui(ui);
        if r.clicked() {
            if matches!(self.variant, ButtonVariant::ToggleChild) {
                ui.ctx().flip_path_enabled(&path);
            }
        }

        ui.reset_style();
        ui.ctx().remove_path();
        r
    }
}
