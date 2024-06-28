use super::*;

pub fn slider(name: &str, value: &mut f32, ui: &mut Ui) {
    ui.label(name);
    ui.spacing_mut().slider_width = ui.available_width() - 60.0;
    Slider::new(value, 0.0..=1.0).max_decimals(2).ui(ui);
    ui.reset_style();
}
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
