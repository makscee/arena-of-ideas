use super::*;

pub struct RectButton {
    size: Option<egui::Vec2>,
    rect: Option<Rect>,
    active: bool,
    enabled: bool,
    color_override: Option<Color32>,
}

impl RectButton {
    pub fn new_size(size: egui::Vec2) -> Self {
        Self {
            size: Some(size),
            rect: None,
            active: false,
            enabled: true,
            color_override: None,
        }
    }
    pub fn new_rect(rect: Rect) -> Self {
        Self {
            size: None,
            rect: Some(rect),
            active: false,
            enabled: true,
            color_override: None,
        }
    }
    pub fn color(mut self, color: Color32) -> Self {
        self.color_override = Some(color);
        self
    }
    pub fn active(mut self, value: bool) -> Self {
        self.active = value;
        self
    }
    pub fn enabled(mut self, value: bool) -> Self {
        self.enabled = value;
        self
    }
    #[must_use]
    pub fn ui(
        self,
        ui: &mut Ui,
        content: impl FnOnce(Color32, Rect, &Response, &mut Ui),
    ) -> Response {
        if ui.any_bar_open() {
            ui.disable();
        }
        let sense = if self.enabled {
            Sense::click()
        } else {
            Sense::hover()
        };
        let response = if let Some(size) = self.size {
            ui.allocate_response(size, sense)
        } else if let Some(rect) = self.rect {
            ui.new_child(UiBuilder::new().max_rect(rect))
                .allocate_rect(rect, sense)
        } else {
            unreachable!()
        };
        let mut rect = response.rect;
        if self.enabled {
            let t = ui.ctx().animate_bool(
                response.id,
                response.hovered() && !response.is_pointer_button_down_on(),
            );
            rect = rect.expand(t * 1.5);
        }
        let color = if self.active {
            YELLOW
        } else if !self.enabled {
            ui.visuals().weak_text_color()
        } else {
            if self.color_override.is_some() && !response.hovered() && response.enabled() {
                self.color_override.unwrap()
            } else {
                ui.style().interact(&response).fg_stroke.color
            }
        };
        let ui = &mut ui.new_child(UiBuilder::new().max_rect(rect));
        content(color, rect, &response, ui);
        response
    }
}
