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
pub fn text_dots_text(text1: Cstr, text2: Cstr, ui: &mut Ui) {
    ui.horizontal(|ui| {
        let rect = ui.available_rect_before_wrap();
        let left_width = text1.label(ui).rect.width();
        let left = rect.left() + left_width + 3.0;
        let right_width = ui
            .with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                text2.label(ui);
            })
            .response
            .rect
            .width();
        let right = rect.right() - 3.0 - right_width;
        let bottom = rect.bottom() - 6.0;
        let line = egui::Shape::dotted_line(
            &[[left, bottom].into(), [right, bottom].into()],
            VISIBLE_LIGHT,
            12.0,
            0.5,
        );
        ui.expand_to_include_x(rect.left() + left_width + right_width + 30.0);
        ui.painter().add(line);
    });
}
pub fn title(text: &str, ui: &mut Ui) {
    text.cstr_cs(VISIBLE_DARK, CstrStyle::Heading2).label(ui);
    br(ui);
}

pub fn cursor_window(ctx: &egui::Context, content: impl FnOnce(&mut Ui)) {
    let mut pos = ctx.pointer_latest_pos().unwrap_or_default();
    const WIDTH: f32 = 350.0;
    let pivot = if pos.x > ctx.screen_rect().right() - WIDTH {
        pos.x -= 10.0;
        Align2::RIGHT_CENTER
    } else {
        pos.x += 10.0;
        Align2::LEFT_CENTER
    };
    Window::new("cursor_window")
        .title_bar(false)
        .frame(Frame::none())
        .max_width(WIDTH)
        .pivot(pivot)
        .fixed_pos(pos)
        .resizable(false)
        .interactable(false)
        .show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                content(ui);
            });
        });
}
