use egui::epaint::CubicBezierShape;

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
pub fn center_window_fullscreen(
    name: &str,
    ctx: &egui::Context,
    add_contents: impl FnOnce(&mut Ui),
) {
    egui::Window::new(name)
        .pivot(Align2::CENTER_CENTER)
        .fixed_pos(ctx.screen_rect().center())
        .title_bar(false)
        .order(Order::Foreground)
        .fixed_size(ctx.screen_rect().shrink(30.0).size())
        .resizable([false, false])
        .show(ctx, |ui| {
            add_contents(ui);
        });
}
pub fn center_window(name: &str, ctx: &egui::Context, add_contents: impl FnOnce(&mut Ui)) {
    egui::Window::new(name)
        .pivot(Align2::CENTER_CENTER)
        .fixed_pos(ctx.screen_rect().center())
        .title_bar(false)
        .order(Order::Foreground)
        .max_width(600.0)
        .resizable([false, false])
        .show(ctx, |ui| {
            ui.set_max_height(ui.ctx().screen_rect().height() * 0.9);
            add_contents(ui);
        });
}
pub fn popup(
    name: &str,
    fullscreen: bool,
    ctx: &egui::Context,
    add_contents: impl FnOnce(&mut Ui),
) {
    let rect = ctx.screen_rect();
    Area::new(Id::new("black_bg"))
        .constrain_to(rect)
        .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .sense(Sense::click())
        .show(ctx, |ui| {
            ui.expand_to_include_rect(rect);
            ui.painter_at(rect)
                .rect_filled(rect, Rounding::ZERO, Color32::from_black_alpha(200));
        });
    if fullscreen {
        center_window_fullscreen(name, ctx, add_contents);
    } else {
        center_window(name, ctx, add_contents);
    }
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
    const WIDTH: f32 = 350.0;
    cursor_window_frame(ctx, Frame::none(), WIDTH, content);
}
pub fn cursor_window_frame(
    ctx: &egui::Context,
    frame: Frame,
    width: f32,
    content: impl FnOnce(&mut Ui),
) {
    let mut pos = ctx.pointer_latest_pos().unwrap_or_default();
    let pivot = if pos.x > ctx.screen_rect().right() - width {
        pos.x -= 10.0;
        Align2::RIGHT_CENTER
    } else {
        pos.x += 10.0;
        Align2::LEFT_CENTER
    };
    egui::Window::new("cursor_window")
        .title_bar(false)
        .frame(frame)
        .max_width(width)
        .pivot(pivot)
        .fixed_pos(pos)
        .resizable(false)
        .interactable(false)
        .order(Order::Tooltip)
        .show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                content(ui);
            });
        });
}
pub fn corners_rounded_rect(rect: Rect, length: f32, stroke: Stroke, ui: &mut Ui) {
    let line = CubicBezierShape::from_points_stroke(
        [
            rect.left_top() + egui::vec2(0.0, length),
            rect.left_top(),
            rect.left_top(),
            rect.left_top() + egui::vec2(length, 0.0),
        ],
        false,
        TRANSPARENT,
        stroke,
    );
    ui.painter().add(line);
    let line = CubicBezierShape::from_points_stroke(
        [
            rect.right_top() + egui::vec2(-length, 0.0),
            rect.right_top(),
            rect.right_top(),
            rect.right_top() + egui::vec2(0.0, length),
        ],
        false,
        TRANSPARENT,
        stroke,
    );
    ui.painter().add(line);
    let line = CubicBezierShape::from_points_stroke(
        [
            rect.right_bottom() + egui::vec2(-length, 0.0),
            rect.right_bottom(),
            rect.right_bottom(),
            rect.right_bottom() + egui::vec2(0.0, -length),
        ],
        false,
        TRANSPARENT,
        stroke,
    );
    ui.painter().add(line);
    let line = CubicBezierShape::from_points_stroke(
        [
            rect.left_bottom() + egui::vec2(length, 0.0),
            rect.left_bottom(),
            rect.left_bottom(),
            rect.left_bottom() + egui::vec2(0.0, -length),
        ],
        false,
        TRANSPARENT,
        stroke,
    );
    ui.painter().add(line);
}
