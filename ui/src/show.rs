use bevy::color::{Color, ColorToPacked};
use egui::{Checkbox, DragValue};

use super::*;

pub trait Show {
    fn show(&self, prefix: Option<&str>, ui: &mut Ui);
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool;
}

impl Show for VarValue {
    fn show(&self, prefix: Option<&str>, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if let Some(prefix) = prefix {
                prefix.cstr_c(VISIBLE_DARK).label(ui);
            }
            match self {
                VarValue::String(v) => v.cstr_s(CstrStyle::Bold),
                VarValue::i32(v) => v.to_string().cstr_s(CstrStyle::Bold),
                VarValue::f32(v) => v.to_string().cstr_s(CstrStyle::Bold),
                VarValue::u64(v) => v.to_string().cstr_s(CstrStyle::Bold),
                VarValue::bool(v) => v.to_string().cstr_s(CstrStyle::Bold),
                VarValue::Vec2(v) => v.to_string().cstr_s(CstrStyle::Bold),
                VarValue::Color(v) => v.to_srgba().to_hex().cstr_cs(v.c32(), CstrStyle::Bold),
            }
            .label(ui)
        });
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        let prefix = prefix.unwrap_or_default();
        match self {
            VarValue::i32(v) => DragValue::new(v).prefix(prefix).ui(ui),
            VarValue::f32(v) => DragValue::new(v).prefix(prefix).ui(ui),
            VarValue::u64(v) => DragValue::new(v).prefix(prefix).ui(ui),
            VarValue::bool(v) => Checkbox::new(v, prefix).ui(ui),
            VarValue::String(v) => Input::new(prefix).ui_string(v, ui),
            VarValue::Vec2(v) => {
                ui.horizontal(|ui| {
                    prefix.to_string().label(ui);
                    let rx = DragValue::new(&mut v.x).prefix("x:").ui(ui);
                    let ry = DragValue::new(&mut v.y).prefix("y:").ui(ui);
                    rx.union(ry)
                })
                .inner
            }
            VarValue::Color(color) => {
                let c = color.to_srgba();
                let mut hsva = ecolor::Hsva::from_srgba_unmultiplied(c.to_u8_array());
                let r = ui.color_edit_button_hsva(&mut hsva);
                *color = Color::hsva(hsva.h, hsva.s, hsva.v, hsva.a);
                r
            }
        }
        .changed()
    }
}
