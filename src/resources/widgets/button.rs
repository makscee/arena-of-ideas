use super::*;

#[derive(Default)]
pub struct Button {
    name: &'static str,
    variant: ButtonVariant,
}

#[derive(Default)]
enum ButtonVariant {
    #[default]
    Click,
    ClickGray,
    ToggleChild,
}

impl Button {
    pub fn click(name: &'static str) -> Self {
        Self { name, ..default() }
    }
    pub fn toggle_child(name: &'static str) -> Self {
        Self {
            name,
            variant: ButtonVariant::ToggleChild,
        }
    }
    pub fn gray(name: &'static str) -> Self {
        Self {
            name,
            variant: ButtonVariant::ClickGray,
        }
    }
    pub fn ui(self, ui: &mut Ui) -> Response {
        ui.ctx().add_path(self.name);
        let path = ui.ctx().path();

        match self.variant {
            ButtonVariant::Click => {}
            ButtonVariant::ClickGray => {
                let style = ui.style_mut();
                style.visuals.widgets.inactive.fg_stroke.color = LIGHT_GRAY;
                style.visuals.widgets.hovered.fg_stroke.color = LIGHT_GRAY;
            }
            ButtonVariant::ToggleChild => {
                if ui.ctx().is_path_enabled(&path) {
                    let style = ui.style_mut();
                    style.visuals.widgets.inactive.weak_bg_fill = DARK_GRAY;
                    style.visuals.widgets.hovered.weak_bg_fill = DARK_GRAY;
                }
            }
        }
        let r = ui.button(self.name);
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
