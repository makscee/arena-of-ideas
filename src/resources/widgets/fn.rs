use super::*;

pub fn br(ui: &mut Ui) {
    ui.horizontal(|ui| {
        let rect = ui.max_rect();
        let line = egui::Shape::dotted_line(
            &[rect.left_center(), rect.right_center()],
            VISIBLE_DARK,
            8.0,
            1.0,
        );
        ui.painter().add(line);
    });
}
pub fn center_window(name: &str, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui)) {
    Window::new(name)
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .title_bar(false)
        .default_width(300.0)
        .resizable([true, false])
        .show(ui.ctx(), add_contents);
}
pub fn text_dots_text(text1: &Cstr, text2: &Cstr, ui: &mut Ui) {
    ui.horizontal(|ui| {
        let rect = ui.available_rect_before_wrap();
        let left = rect.left() + text1.label(ui).rect.width() + 3.0;
        let right = rect.right()
            - 3.0
            - ui.with_layout(Layout::right_to_left(egui::Align::Min), |ui| {
                text2.label(ui);
            })
            .response
            .rect
            .width();
        let bottom = rect.bottom() - 6.0;
        let line = egui::Shape::dotted_line(
            &[[left, bottom].into(), [right, bottom].into()],
            VISIBLE_LIGHT,
            8.0,
            0.5,
        );
        ui.painter().add(line);
    });
}
pub fn title(text: &str, ui: &mut Ui) {
    text.cstr_cs(VISIBLE_DARK, CstrStyle::Heading2).label(ui);
    br(ui);
}
