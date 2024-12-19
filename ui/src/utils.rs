use bevy::{
    color::{Color, ColorToPacked, LinearRgba},
    input::ButtonInput,
    log::debug,
    math::{vec2, Vec2, Vec3},
    prelude::{Camera, GlobalTransform, MouseButton},
};
use bevy_egui::EguiContext;
use egui::{epaint::PathShape, Pos2, TextureId};

use super::*;

pub fn debug_rect(rect: Rect, ctx: &egui::Context) {
    ctx.debug_painter().rect(
        rect,
        Rounding::ZERO,
        YELLOW_DARK.gamma_multiply(0.5),
        Stroke {
            width: 1.0,
            color: YELLOW,
        },
    );
}
pub fn debug_available_rect(ui: &mut Ui) {
    debug_rect(ui.available_rect_before_wrap(), ui.ctx());
}
