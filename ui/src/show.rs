use bevy::{
    color::{Color, ColorToPacked},
    math::Vec2,
};
use ecolor::Hsva;
use egui::{Checkbox, DragValue};

use super::*;

pub trait Show {
    fn show(&self, prefix: Option<&str>, ui: &mut Ui);
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool;
}

impl Show for VarName {
    fn show(&self, prefix: Option<&str>, ui: &mut Ui) {
        format!("{}{}", prefix.unwrap_or_default(), self.cstr_expanded()).label(ui);
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        Selector::new(prefix.unwrap_or_default()).ui_enum(self, ui)
    }
}
impl Show for VarValue {
    fn show(&self, prefix: Option<&str>, ui: &mut Ui) {
        ui.horizontal(|ui| match self {
            VarValue::String(v) => v.show(prefix, ui),
            VarValue::i32(v) => v.show(prefix, ui),
            VarValue::f32(v) => v.show(prefix, ui),
            VarValue::u64(v) => v.show(prefix, ui),
            VarValue::bool(v) => v.show(prefix, ui),
            VarValue::Vec2(v) => v.show(prefix, ui),
            VarValue::Color(v) => v.show(prefix, ui),
        });
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        ui.horizontal(|ui| match self {
            VarValue::i32(v) => v.show_mut(prefix, ui),
            VarValue::f32(v) => v.show_mut(prefix, ui),
            VarValue::u64(v) => v.show_mut(prefix, ui),
            VarValue::bool(v) => v.show_mut(prefix, ui),
            VarValue::String(v) => v.show_mut(prefix, ui),
            VarValue::Vec2(v) => v.show_mut(prefix, ui),
            VarValue::Color(v) => v.show_mut(prefix, ui),
        })
        .inner
    }
}

impl Show for i32 {
    fn show(&self, prefix: Option<&str>, ui: &mut Ui) {
        format!("{}{}", prefix.unwrap_or_default(), self.cstr_expanded()).label(ui);
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        ui.horizontal(|ui| {
            if let Some(prefix) = prefix {
                prefix.cstr().label(ui);
            }
            DragValue::new(self).ui(ui).changed()
        })
        .inner
    }
}
impl Show for f32 {
    fn show(&self, prefix: Option<&str>, ui: &mut Ui) {
        format!("{}{}", prefix.unwrap_or_default(), self.cstr_expanded()).label(ui);
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        ui.horizontal(|ui| {
            if let Some(prefix) = prefix {
                prefix.cstr().label(ui);
            }
            DragValue::new(self).ui(ui).changed()
        })
        .inner
    }
}
impl Show for u64 {
    fn show(&self, prefix: Option<&str>, ui: &mut Ui) {
        format!("{}{}", prefix.unwrap_or_default(), self.cstr_expanded()).label(ui);
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        ui.horizontal(|ui| {
            if let Some(prefix) = prefix {
                prefix.cstr().label(ui);
            }
            DragValue::new(self).ui(ui).changed()
        })
        .inner
    }
}
impl Show for bool {
    fn show(&self, prefix: Option<&str>, ui: &mut Ui) {
        format!("{}{}", prefix.unwrap_or_default(), self.cstr_expanded()).label(ui);
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        ui.horizontal(|ui| {
            Checkbox::new(self, prefix.unwrap_or_default().to_owned().widget(1.0, ui))
                .ui(ui)
                .changed()
        })
        .inner
    }
}
impl Show for Vec2 {
    fn show(&self, prefix: Option<&str>, ui: &mut Ui) {
        format!("{}{}", prefix.unwrap_or_default(), self.cstr_expanded()).label(ui);
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        ui.horizontal(|ui| {
            if let Some(prefix) = prefix {
                prefix.cstr().label(ui);
            }
            let rx = DragValue::new(&mut self.x).prefix("x:").ui(ui);
            let ry = DragValue::new(&mut self.y).prefix("y:").ui(ui);
            rx.union(ry)
        })
        .inner
        .changed()
    }
}
impl Show for String {
    fn show(&self, prefix: Option<&str>, ui: &mut Ui) {
        format!("{}{self}", prefix.unwrap_or_default()).label(ui);
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        Input::new(prefix.unwrap_or_default())
            .ui_string(self, ui)
            .changed()
    }
}
impl Show for Color {
    fn show(&self, prefix: Option<&str>, ui: &mut Ui) {
        format!("{}{}", prefix.unwrap_or_default(), self.cstr_expanded()).label(ui);
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        ui.horizontal(|ui| {
            let mut c = self.c32();
            if c.show_mut(prefix, ui) {
                *self = c.to_color();
                true
            } else {
                false
            }
        })
        .inner
    }
}
impl Show for Color32 {
    fn show(&self, prefix: Option<&str>, ui: &mut Ui) {
        format!("{}{}", prefix.unwrap_or_default(), self.cstr_expanded()).label(ui);
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        ui.horizontal(|ui| {
            if let Some(prefix) = prefix {
                prefix.cstr().label(ui);
            }
            let mut hsva = self.clone().into();
            if ui.color_edit_button_hsva(&mut hsva).changed() {
                *self = hsva.into();
                true
            } else {
                false
            }
        })
        .inner
    }
}
