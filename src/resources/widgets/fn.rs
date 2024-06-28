use super::*;

pub fn br(ui: &mut Ui) {
    ui.horizontal(|ui| {
        let rect = ui.max_rect();
        let line = egui::Shape::dotted_line(
            &[rect.left_center(), rect.right_center()],
            DARK_GRAY,
            8.0,
            1.5,
        );
        ui.painter().add(line);
    });
}
