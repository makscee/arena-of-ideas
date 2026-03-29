use bevy::prelude::*;
use bevy_egui::egui::{self, Color32, Rect, Stroke, Ui};

/// Painter action enum — defines unit visual appearance.
#[derive(Debug, Clone)]
pub enum PainterAction {
    Circle { radius: f32 },
    Rectangle { width: f32, height: f32 },
    Hollow { width: f32 },
    Solid,
    Translate { x: f32, y: f32 },
    Color { r: u8, g: u8, b: u8, a: u8 },
    Alpha { alpha: f32 },
    Paint,
}

/// Simple painter that uses egui's public API.
pub struct UnitPainter {
    pub center: egui::Pos2,
    pub size: f32,
    pub color: Color32,
    pub hollow: Option<f32>,
    pub offset: egui::Vec2,
}

impl UnitPainter {
    pub fn new(rect: Rect) -> Self {
        Self {
            center: rect.center(),
            size: rect.width().min(rect.height()) * 0.5,
            color: Color32::WHITE,
            hollow: None,
            offset: egui::Vec2::ZERO,
        }
    }

    pub fn execute(&mut self, actions: &[PainterAction], ui: &Ui) {
        for action in actions {
            match action {
                PainterAction::Circle { radius } => {
                    let r = radius * self.size;
                    let pos = self.center + self.offset;
                    if let Some(w) = self.hollow {
                        ui.painter().circle_stroke(pos, r, Stroke::new(w, self.color));
                    } else {
                        ui.painter().circle_filled(pos, r, self.color);
                    }
                }
                PainterAction::Rectangle { width, height } => {
                    let w = width * self.size;
                    let h = height * self.size;
                    let pos = self.center + self.offset;
                    let rect = Rect::from_center_size(pos, egui::vec2(w * 2.0, h * 2.0));
                    if let Some(sw) = self.hollow {
                        ui.painter().rect_stroke(rect, 0.0, Stroke::new(sw, self.color), egui::StrokeKind::Middle);
                    } else {
                        ui.painter().rect_filled(rect, 0.0, self.color);
                    }
                }
                PainterAction::Hollow { width } => {
                    self.hollow = if *width > 0.0 { Some(*width) } else { None };
                }
                PainterAction::Solid => {
                    self.hollow = None;
                }
                PainterAction::Translate { x, y } => {
                    self.offset += egui::vec2(*x * self.size, *y * self.size);
                }
                PainterAction::Color { r, g, b, a } => {
                    self.color = Color32::from_rgba_premultiplied(*r, *g, *b, *a);
                }
                PainterAction::Alpha { alpha } => {
                    self.color = self.color.gamma_multiply(alpha.clamp(0.0, 1.0));
                }
                PainterAction::Paint => {
                    // Reset offset for next group
                    self.offset = egui::Vec2::ZERO;
                }
            }
        }
    }
}

/// Paint a unit with HP/PWR indicators and name.
pub fn paint_default_unit(
    rect: Rect,
    color: Color32,
    hp: i32,
    pwr: i32,
    dmg: i32,
    name: &str,
    ui: &mut Ui,
) {
    let center = rect.center();
    let up = rect.width().min(rect.height()) * 0.5;
    let painter = ui.painter();

    // Outer ring
    painter.circle_stroke(
        center,
        up * 0.95,
        Stroke::new(2.0, Color32::from_rgba_premultiplied(
            color.r().saturating_add(50),
            color.g().saturating_add(50),
            color.b().saturating_add(50),
            200,
        )),
    );

    // Inner filled circle
    painter.circle_filled(center, up * 0.85, color);

    // HP indicator (bottom-right)
    let hp_pos = center + egui::vec2(up * 0.55, up * 0.55);
    painter.circle_filled(hp_pos, up * 0.22, Color32::from_rgb(30, 30, 30));
    painter.circle_stroke(hp_pos, up * 0.22, Stroke::new(1.5, Color32::from_rgb(213, 0, 0)));
    let hp_color = if dmg > 0 { Color32::from_rgb(255, 80, 80) } else { Color32::WHITE };
    painter.text(
        hp_pos,
        egui::Align2::CENTER_CENTER,
        format!("{}", hp - dmg),
        egui::FontId::proportional(up * 0.25),
        hp_color,
    );

    // PWR indicator (bottom-left)
    let pwr_pos = center + egui::vec2(-up * 0.55, up * 0.55);
    painter.circle_filled(pwr_pos, up * 0.22, Color32::from_rgb(30, 30, 30));
    painter.circle_stroke(pwr_pos, up * 0.22, Stroke::new(1.5, Color32::from_rgb(255, 145, 0)));
    painter.text(
        pwr_pos,
        egui::Align2::CENTER_CENTER,
        format!("{}", pwr),
        egui::FontId::proportional(up * 0.25),
        Color32::WHITE,
    );

    // Name above
    painter.text(
        center + egui::vec2(0.0, -up * 0.95),
        egui::Align2::CENTER_CENTER,
        name,
        egui::FontId::proportional(up * 0.22),
        Color32::WHITE,
    );
}
