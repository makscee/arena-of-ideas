use super::*;

pub struct RectButton {
    size: Option<egui::Vec2>,
    rect: Option<Rect>,
    active: bool,
    color_override: Option<Color32>,
}

impl RectButton {
    pub fn new_size(size: egui::Vec2) -> Self {
        Self {
            size: Some(size),
            rect: None,
            active: false,
            color_override: None,
        }
    }
    pub fn new_rect(rect: Rect) -> Self {
        Self {
            size: None,
            rect: Some(rect),
            active: false,
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
    #[must_use]
    pub fn ui(self, ui: &mut Ui, content: impl FnOnce(Color32, Rect, &mut Ui)) -> Response {
        let response = if let Some(size) = self.size {
            ui.allocate_response(size, Sense::click())
        } else if let Some(rect) = self.rect {
            ui.new_child(UiBuilder::new().max_rect(rect))
                .allocate_rect(rect, Sense::click())
        } else {
            unreachable!()
        };
        let mut rect = response.rect;
        if response.hovered() {
            rect = rect.expand(1.0);
        }
        let color = if self.active {
            YELLOW
        } else {
            if self.color_override.is_some() && !response.hovered() && response.enabled() {
                self.color_override.unwrap()
            } else {
                ui.style().interact(&response).fg_stroke.color
            }
        };
        let ui = &mut ui.new_child(UiBuilder::new().max_rect(rect));
        content(color, rect, ui);
        response
    }
}
