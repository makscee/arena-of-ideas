use bevy_egui::egui::ScrollArea;
use ecolor::linear_u8_from_linear_f32;
use egui::{CornerRadius, Margin, Shadow};

use super::*;

#[macro_export]
macro_rules! hex_color_noa {
    ($s:literal) => {{
        let array = color_hex::color_from_hex!($s);
        Color32::from_rgb(array[0], array[1], array[2])
    }};
}

pub const YELLOW: Color32 = hex_color_noa!("#D98F00");
pub const ORANGE: Color32 = hex_color_noa!("#DC6814");
pub const RED: Color32 = hex_color_noa!("#DC143C");
pub const DARK_RED: Color32 = hex_color_noa!("#9D0E2B");
pub const GREEN: Color32 = hex_color_noa!("#64DD17");
pub const PURPLE: Color32 = hex_color_noa!("#B50DA4");
pub const LIGHT_PURPLE: Color32 = hex_color_noa!("#95408D");
pub const CYAN: Color32 = hex_color_noa!("#00B8D4");
pub const GRAY: Color32 = hex_color_noa!("#616161");

pub const MISSING_COLOR: Color32 = hex_color_noa!("#FF00FF");
pub const TRANSPARENT: Color32 = Color32::TRANSPARENT;
pub const CREDITS_SYM: char = '¤';

pub const SHADOW: Shadow = Shadow {
    offset: [8, 8],
    blur: 0,
    spread: 0,
    color: Color32::from_rgba_premultiplied(20, 20, 20, 35),
};
pub const MARGIN: Margin = Margin::same(8);

pub const ROUNDING: CornerRadius = CornerRadius::same(6);

pub const LINE_HEIGHT: f32 = 22.0;
pub const UNIT_SIZE: f32 = 1.0;

pub fn dark_frame() -> Frame {
    Frame {
        inner_margin: Margin::same(5),
        outer_margin: Margin::same(5),
        corner_radius: ROUNDING,
        shadow: SHADOW,
        fill: subtle_background(),
        stroke: subtle_borders_and_separators().stroke(),
    }
}

pub trait ToStroke {
    fn stroke(self) -> Stroke;
    fn stroke_w(self, width: f32) -> Stroke;
}

impl ToStroke for Color32 {
    fn stroke(self) -> Stroke {
        Stroke::new(1.0, self)
    }
    fn stroke_w(self, width: f32) -> Stroke {
        Stroke::new(width, self)
    }
}

pub trait ColorAlphaExt {
    fn alpha(self, a: f32) -> Self;
}

impl ColorAlphaExt for Color32 {
    fn alpha(self, a: f32) -> Self {
        Color32::from_rgba_premultiplied(self.r(), self.g(), self.b(), linear_u8_from_linear_f32(a))
    }
}

pub trait ErrorExt {
    fn ui(self, ui: &mut Ui);
}

impl<T> ErrorExt for Result<T, NodeError> {
    #[track_caller]
    fn ui(self, ui: &mut Ui) {
        if let Err(e) = self {
            e.ui(ui);
        }
    }
}

impl ErrorExt for NodeError {
    fn ui(self, ui: &mut Ui) {
        let error_text = format!("{}\n[s {}]", self.cstr(), std::panic::Location::caller());
        error_text.label_w(ui);
        // error_text.clone().button(ui).bar_menu(|ui| {
        //     ScrollArea::vertical().show(ui, |ui| {
        //         if let Some(mut b) = self.bt {
        //             b.resolve();
        //             error_text.cstr().label(ui);
        //             format!("[s {b:?}]").label(ui);
        //         }
        //     });
        // });
    }
}
