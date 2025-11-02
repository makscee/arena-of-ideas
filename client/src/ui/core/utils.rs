use super::*;

pub fn debug_rect(rect: Rect, ctx: &egui::Context) {
    ctx.debug_painter().rect(
        rect,
        CornerRadius::ZERO,
        YELLOW.gamma_multiply(0.25),
        Stroke {
            width: 1.0,
            color: YELLOW.gamma_multiply(0.7),
        },
        egui::StrokeKind::Middle,
    );
}
pub fn debug_available_rect(ui: &mut Ui) {
    debug_rect(ui.available_rect_before_wrap(), ui.ctx());
}
