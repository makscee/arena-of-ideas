use bevy::{
    color::Color,
    math::{vec2, Vec2},
};
use bevy_egui::egui::{self, Checkbox, Color32, DragValue, Ui, Widget};
use ui::*;
use utils::default;

use super::*;

pub trait Show {
    fn show(&self, prefix: Option<&str>, context: &Context, ui: &mut Ui);
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool;
}

impl Show for VarName {
    fn show(&self, prefix: Option<&str>, _: &Context, ui: &mut Ui) {
        format!("{}{}", prefix.unwrap_or_default(), self.cstr_expanded()).label(ui);
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        Selector::new(prefix.unwrap_or_default()).ui_enum(self, ui)
    }
}
impl Show for VarValue {
    fn show(&self, prefix: Option<&str>, context: &Context, ui: &mut Ui) {
        ui.horizontal(|ui| match self {
            VarValue::String(v) => v.show(prefix, context, ui),
            VarValue::i32(v) => v.show(prefix, context, ui),
            VarValue::f32(v) => v.show(prefix, context, ui),
            VarValue::u64(v) => v.show(prefix, context, ui),
            VarValue::bool(v) => v.show(prefix, context, ui),
            VarValue::Vec2(v) => v.show(prefix, context, ui),
            VarValue::Color32(v) => v.show(prefix, context, ui),
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
            VarValue::Color32(v) => v.show_mut(prefix, ui),
        })
        .inner
    }
}

impl Show for i32 {
    fn show(&self, prefix: Option<&str>, _: &Context, ui: &mut Ui) {
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
    fn show(&self, prefix: Option<&str>, _: &Context, ui: &mut Ui) {
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
    fn show(&self, prefix: Option<&str>, _: &Context, ui: &mut Ui) {
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
    fn show(&self, prefix: Option<&str>, _: &Context, ui: &mut Ui) {
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
    fn show(&self, prefix: Option<&str>, _: &Context, ui: &mut Ui) {
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
    fn show(&self, prefix: Option<&str>, _: &Context, ui: &mut Ui) {
        format!("{}{self}", prefix.unwrap_or_default()).label(ui);
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        Input::new(prefix.unwrap_or_default())
            .ui_string(self, ui)
            .changed()
    }
}
impl Show for Color {
    fn show(&self, prefix: Option<&str>, _: &Context, ui: &mut Ui) {
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
    fn show(&self, prefix: Option<&str>, _: &Context, ui: &mut Ui) {
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

impl Show for Expression {
    fn show(&self, prefix: Option<&str>, context: &Context, ui: &mut Ui) {
        format!(
            "{}{}{}",
            prefix.unwrap_or_default(),
            self.cstr_expanded(),
            self.get_value(context).unwrap_or_default()
        )
        .label(ui);
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        CollapsingSelector::ui(self, prefix, ui, |v, ui| match v {
            Expression::One | Expression::Zero | Expression::GT => false,
            Expression::Var(v) => v.show_mut(Some("v:"), ui),
            Expression::V(v) => v.show_mut(Some("v:"), ui),
            Expression::S(v) => v.show_mut(Some("v:"), ui),
            Expression::F(v) => v.show_mut(Some("v:"), ui),
            Expression::I(v) => v.show_mut(Some("v:"), ui),
            Expression::B(v) => v.show_mut(Some("v:"), ui),
            Expression::C(v) => v.show_mut(Some("v:"), ui),
            Expression::V2(x, y) => {
                let mut v = vec2(*x, *y);
                if v.show_mut(Some("v:"), ui) {
                    *x = v.x;
                    *y = v.y;
                    true
                } else {
                    false
                }
            }
            Expression::Sin(x)
            | Expression::Cos(x)
            | Expression::Even(x)
            | Expression::Abs(x)
            | Expression::Floor(x)
            | Expression::Ceil(x)
            | Expression::Fract(x)
            | Expression::Sqr(x) => x.show_mut(Some("x:"), ui),
            Expression::Macro(a, b)
            | Expression::Sum(a, b)
            | Expression::Sub(a, b)
            | Expression::Mul(a, b)
            | Expression::Div(a, b)
            | Expression::Max(a, b)
            | Expression::Min(a, b)
            | Expression::Mod(a, b)
            | Expression::And(a, b)
            | Expression::Or(a, b)
            | Expression::Equals(a, b)
            | Expression::GreaterThen(a, b)
            | Expression::LessThen(a, b) => {
                let mut r = false;
                ui.vertical(|ui| {
                    r |= a.show_mut(Some("a:".into()), ui);
                    r |= b.show_mut(Some("b:".into()), ui);
                });
                r
            }
            Expression::If(i, t, e) => {
                let mut r = false;
                ui.vertical(|ui| {
                    r |= i.show_mut(Some("if:".into()), ui);
                    r |= t.show_mut(Some("then:".into()), ui);
                    r |= e.show_mut(Some("else:".into()), ui);
                });
                r
            }
        })
    }
}

impl Show for PainterAction {
    fn show(&self, prefix: Option<&str>, context: &Context, ui: &mut Ui) {
        let mut s = String::new();
        for i in <Self as Injector<Expression>>::get_inner(self) {
            match i.get_value(context) {
                Ok(v) => s += &v.cstr(),
                Err(e) => s += &e.to_string(),
            }
        }
        format!("{s}{}{}", prefix.unwrap_or_default(), self.cstr_expanded()).label_w(ui);
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        CollapsingSelector::ui(self, prefix, ui, |v, ui| match v {
            PainterAction::Circle(x)
            | PainterAction::Rectangle(x)
            | PainterAction::Text(x)
            | PainterAction::Hollow(x)
            | PainterAction::Translate(x)
            | PainterAction::Rotate(x)
            | PainterAction::Scale(x)
            | PainterAction::Alpha(x)
            | PainterAction::Color(x) => x.show_mut(Some("x:"), ui),
            PainterAction::Repeat(x, a) => {
                let mut r = false;
                ui.vertical(|ui| {
                    r |= x.show_mut(Some("cnt:"), ui);
                    r |= a.show_mut(Some("a:"), ui);
                });
                r
            }
            PainterAction::List(l) => {
                let mut r = false;
                ui.vertical(|ui| {
                    for (i, a) in l.iter_mut().enumerate() {
                        ui.push_id(i, |ui| {
                            r |= a.show_mut(None, ui);
                        });
                    }
                });
                r
            }
            PainterAction::Paint => false,
        })
    }
}
impl Show for Material {
    fn show(&self, prefix: Option<&str>, context: &Context, ui: &mut Ui) {
        if let Some(prefix) = prefix {
            prefix.cstr().label(ui);
        }
        for i in &self.0 {
            i.show(None, context, ui);
        }
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        if let Some(prefix) = prefix {
            prefix.cstr().label(ui);
        }
        let mut changed = false;
        for (i, a) in self.0.iter_mut().enumerate() {
            changed |= a.show_mut(Some(&i.to_string()), ui);
        }
        if "+"
            .cstr_cs(VISIBLE_BRIGHT, CstrStyle::Bold)
            .button(ui)
            .clicked()
        {
            self.0.push(default());
        }
        changed
    }
}
impl Show for Trigger {
    fn show(&self, prefix: Option<&str>, _: &Context, ui: &mut Ui) {
        if let Some(prefix) = prefix {
            prefix.cstr_c(VISIBLE_DARK).label(ui);
        }
        self.cstr_cs(CYAN, CstrStyle::Bold).label(ui);
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        Selector::new(prefix.unwrap_or_default()).ui_enum(self, ui)
    }
}
