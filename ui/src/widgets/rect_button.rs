use super::*;

pub struct RectButton {
    size: egui::Vec2,
    active: bool,
}

impl RectButton {
    pub fn new(size: egui::Vec2) -> Self {
        Self {
            size,
            active: false,
        }
    }
    pub fn active(mut self, value: bool) -> Self {
        self.active = value;
        self
    }
    #[must_use]
    pub fn ui(self, ui: &mut Ui, content: impl FnOnce(Color32, Rect, &mut Ui)) -> Response {
        let response = ui.allocate_response(self.size, Sense::click());
        let mut rect = response.rect;
        if response.hovered() {
            rect = rect.expand(1.0);
        }
        let color = if self.active {
            YELLOW
        } else {
            ui.style().interact(&response).fg_stroke.color
        };
        let ui = &mut ui.new_child(UiBuilder::new().max_rect(rect));
        content(color, rect, ui);
        response
    }
}
