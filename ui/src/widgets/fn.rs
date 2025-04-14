use egui::epaint::CubicBezierShape;

use super::*;

pub fn br(ui: &mut Ui) {
    ui.horizontal(|ui| {
        let rect = ui.max_rect();
        ui.painter().line_segment(
            [rect.left_top(), rect.right_top()],
            Stroke {
                width: 1.0,
                color: tokens_global().subtle_borders_and_separators(),
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
            ui.painter_at(rect).rect_filled(
                rect,
                CornerRadius::ZERO,
                Color32::from_black_alpha(200),
            );
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
            tokens_global().subtle_borders_and_separators(),
            12.0,
            0.5,
        );
        ui.expand_to_include_x(rect.left() + left_width + right_width + 30.0);
        ui.painter().add(line);
    });
}
pub fn title(text: &str, ui: &mut Ui) {
    text.cstr_s(CstrStyle::Heading2).label(ui);
    br(ui);
}

pub fn cursor_window(ctx: &egui::Context, content: impl FnOnce(&mut Ui)) {
    const WIDTH: f32 = 350.0;
    cursor_window_frame(ctx, Frame::new(), WIDTH, content);
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

pub fn show_slot(i: usize, slots: usize, bottom: bool, ui: &mut Ui) -> Response {
    let full_rect = ui.available_rect_before_wrap();
    let rect = slot_rect(i.at_most(slots - 1), slots, full_rect, bottom);
    let mut cui = ui.new_child(UiBuilder::new().max_rect(rect));
    if ui.any_bar_open() {
        cui.disable();
    }
    let r = cui.allocate_rect(rect, Sense::click_and_drag());
    let mut stroke = Stroke::new(
        1.0,
        if r.hovered() {
            tokens_global().hovered_ui_element_border()
        } else {
            tokens_global().ui_element_border_and_focus_rings()
        },
    );
    let t = ui.ctx().animate_bool(r.id, r.hovered());
    let length = egui::emath::lerp(15.0..=20.0, t);
    stroke.width += t;
    corners_rounded_rect(r.rect.shrink(3.0), length, stroke, ui);
    r
}
pub fn slot_rect(i: usize, slots: usize, full_rect: Rect, bottom: bool) -> Rect {
    let pos_i = i as i32 as f32;
    let size = (full_rect.width() / slots as f32).at_most(full_rect.height());
    let rect = if bottom {
        let lb = full_rect.left_bottom();
        Rect::from_two_pos(lb, lb + egui::vec2(size, -size))
    } else {
        let lt = full_rect.left_top();
        Rect::from_two_pos(lt, lt + egui::vec2(size, size))
    };
    rect.translate(egui::vec2(size * pos_i, 0.0))
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
pub fn close_btn(rect: Rect, ui: &mut Ui) -> Response {
    let close_btn_size = egui::vec2(12.0, 12.0);
    let close_btn_rect = egui::Align2::RIGHT_CENTER.align_size_within_rect(close_btn_size, rect);

    let close_btn_id = ui.auto_id_with("tab_close_btn");
    let close_btn_response = ui
        .interact(close_btn_rect, close_btn_id, Sense::click_and_drag())
        .on_hover_cursor(egui::CursorIcon::Default);

    let visuals = ui.style().interact(&close_btn_response);
    let rect = close_btn_rect.shrink(2.0).expand(visuals.expansion);
    let stroke = visuals.fg_stroke;
    ui.painter()
        .line_segment([rect.left_top(), rect.right_bottom()], stroke);
    ui.painter()
        .line_segment([rect.right_top(), rect.left_bottom()], stroke);
    close_btn_response
}
