use super::*;

pub struct Middle3 {
    width: f32,
    height: f32,
}

impl Default for Middle3 {
    fn default() -> Self {
        Self {
            width: 150.0,
            height: 25.0,
        }
    }
}

impl Middle3 {
    pub fn width(mut self, value: f32) -> Self {
        self.width = value;
        self
    }
    pub fn height(mut self, value: f32) -> Self {
        self.height = value;
        self
    }
    pub fn ui(self, ui: &mut Ui, add_contents: impl FnOnce(&mut [Ui; 3])) {
        let full_rect = Rect::from_min_size(
            ui.cursor().left_top(),
            egui::vec2(ui.available_width(), self.height),
        );
        let side_width = (full_rect.width() - self.width) * 0.5;
        let rect_center =
            Rect::from_center_size(full_rect.center(), egui::vec2(self.width, self.height));
        let rect_left = Rect::from_min_size(full_rect.min, egui::vec2(side_width, self.height));
        let rect_right =
            Rect::from_min_size(rect_center.right_top(), egui::vec2(side_width, self.height));
        let mut uis = [
            ui.child_ui(rect_left, Layout::right_to_left(Align::Center)),
            ui.child_ui(
                rect_center,
                Layout::centered_and_justified(egui::Direction::TopDown),
            ),
            ui.child_ui(rect_right, Layout::left_to_right(Align::Center)),
        ];
        add_contents(&mut uis);
        ui.advance_cursor_after_rect(full_rect);
    }
}
