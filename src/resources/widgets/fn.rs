use super::*;

pub fn br(ui: &mut Ui) {
    ui.horizontal(|ui| {
        let rect = ui.max_rect();
        ui.painter().line_segment(
            [rect.left_top(), rect.right_top()],
            Stroke {
                width: 1.0,
                color: VISIBLE_DARK,
            },
        );
    });
}
pub fn space(ui: &mut Ui) {
    ui.add_space(13.0);
}
pub fn center_window(name: &str, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui)) {
    Window::new(name)
        .pivot(Align2::CENTER_CENTER)
        .fixed_pos(ui.clip_rect().center())
        .constrain(false)
        .title_bar(false)
        .default_width(300.0)
        .resizable([false, false])
        .show(ui.ctx(), add_contents);
}
pub fn popup(name: &str, ctx: &egui::Context, add_contents: impl FnOnce(&mut Ui)) {
    let rect = ctx.screen_rect();
    CentralPanel::default()
        .frame(Frame::none())
        .show(ctx, |ui| {
            ui.allocate_rect(rect, Sense::click_and_drag());
            ui.painter_at(rect)
                .rect_filled(rect, Rounding::ZERO, Color32::from_black_alpha(180));
            center_window(name, ui, add_contents);
        });
}
pub fn text_dots_text(text1: &Cstr, text2: &Cstr, ui: &mut Ui) {
    ui.horizontal(|ui| {
        let rect = ui.available_rect_before_wrap();
        let left = rect.left() + text1.label(ui).rect.width() + 3.0;
        let right = rect.right()
            - 3.0
            - ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                text2.label(ui);
            })
            .response
            .rect
            .width();
        let bottom = rect.bottom() - 6.0;
        let line = egui::Shape::dotted_line(
            &[[left, bottom].into(), [right, bottom].into()],
            VISIBLE_LIGHT,
            12.0,
            0.5,
        );
        ui.painter().add(line);
    });
}
pub fn title(text: &str, ui: &mut Ui) {
    text.cstr_cs(VISIBLE_DARK, CstrStyle::Heading2).label(ui);
}
