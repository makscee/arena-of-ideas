use serde::de::DeserializeOwned;

use super::*;

pub trait ShowPrefix {
    fn show(&self, ui: &mut Ui);
}
impl ShowPrefix for Option<&str> {
    fn show(&self, ui: &mut Ui) {
        if let Some(s) = self {
            s.cstr_cs(VISIBLE_DARK, CstrStyle::Small).label(ui);
        }
    }
}
pub trait Show: StringData {
    fn show(&self, prefix: Option<&str>, context: &Context, ui: &mut Ui);
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool;
}

impl<T> Show for Vec<Box<T>>
where
    T: Show + Default + Serialize + DeserializeOwned,
{
    fn show(&self, prefix: Option<&str>, context: &Context, ui: &mut Ui) {
        prefix.show(ui);
        for (i, v) in self.into_iter().enumerate() {
            v.show(Some(&format!("[vd {i}:]")), context, ui);
        }
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        prefix.show(ui);
        let mut changed = false;
        let mut swap = None;
        let mut delete = None;
        let mut insert = None;
        let len = self.len();
        fn plus_btn(ui: &mut Ui) -> bool {
            "+".cstr_cs(VISIBLE_BRIGHT, CstrStyle::Bold)
                .button(ui)
                .clicked()
        }
        for (i, a) in self.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        if i > 0 && "<".cstr_s(CstrStyle::Bold).button(ui).clicked() {
                            swap = Some((i, i - 1));
                        }
                        if i + 1 < len && ">".cstr_s(CstrStyle::Bold).button(ui).clicked() {
                            swap = Some((i, i + 1));
                        }
                    });
                    ui.horizontal(|ui| {
                        if "-".cstr_cs(RED, CstrStyle::Bold).button(ui).clicked() {
                            delete = Some(i);
                        }
                        if plus_btn(ui) {
                            insert = Some(i + 1);
                        }
                    });
                });
                changed |= a.show_mut(Some(&i.to_string()), ui);
            });
        }
        if self.is_empty() && plus_btn(ui) {
            insert = Some(0);
        }
        if let Some(delete) = delete {
            changed = true;
            self.remove(delete);
        }
        if let Some(index) = insert {
            changed = true;
            self.insert(index, default());
        }
        if let Some((a, b)) = swap {
            changed = true;
            self.swap(a, b);
        }
        changed
    }
}
impl Show for VarName {
    fn show(&self, prefix: Option<&str>, _: &Context, ui: &mut Ui) {
        prefix.show(ui);
        self.cstr_expanded().label(ui);
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
            VarValue::Entity(v) => Entity::from_bits(*v).show(prefix, context, ui),
        })
        .inner
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
            VarValue::Entity(v) => Entity::from_bits(*v).show_mut(prefix, ui),
        })
        .inner
    }
}

impl Show for i32 {
    fn show(&self, prefix: Option<&str>, _: &Context, ui: &mut Ui) {
        prefix.show(ui);
        self.cstr().label(ui);
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        ui.horizontal(|ui| {
            if let Some(prefix) = prefix {
                prefix.cstr().label(ui);
            }
            DragValue::new(self).ui(ui)
        })
        .inner
        .changed()
    }
}
impl Show for f32 {
    fn show(&self, prefix: Option<&str>, _: &Context, ui: &mut Ui) {
        prefix.show(ui);
        self.cstr().label(ui);
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        ui.horizontal(|ui| {
            prefix.show(ui);
            DragValue::new(self).min_decimals(1).ui(ui)
        })
        .inner
        .changed()
    }
}
impl Show for u64 {
    fn show(&self, prefix: Option<&str>, _: &Context, ui: &mut Ui) {
        prefix.show(ui);
        self.cstr().label(ui);
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        ui.horizontal(|ui| {
            if let Some(prefix) = prefix {
                prefix.cstr().label(ui);
            }
            DragValue::new(self).ui(ui)
        })
        .inner
        .changed()
    }
}
impl Show for bool {
    fn show(&self, prefix: Option<&str>, _: &Context, ui: &mut Ui) {
        prefix.show(ui);
        self.cstr().label(ui);
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        ui.horizontal(|ui| {
            Checkbox::new(self, prefix.unwrap_or_default().to_owned().widget(1.0, ui)).ui(ui)
        })
        .inner
        .changed()
    }
}
impl Show for Vec2 {
    fn show(&self, prefix: Option<&str>, _: &Context, ui: &mut Ui) {
        prefix.show(ui);
        self.cstr().label(ui);
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
        prefix.show(ui);
        self.cstr().label(ui);
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        Input::new(prefix.unwrap_or_default())
            .ui_string(self, ui)
            .changed()
    }
}
impl Show for Color {
    fn show(&self, prefix: Option<&str>, _: &Context, ui: &mut Ui) {
        prefix.show(ui);
        self.cstr().label(ui);
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        ui.horizontal(|ui| {
            let mut c = self.c32();
            let r = c.show_mut(prefix, ui);
            if r {
                *self = c.to_color();
            }
            r
        })
        .inner
    }
}
impl Show for Color32 {
    fn show(&self, prefix: Option<&str>, _: &Context, ui: &mut Ui) {
        prefix.show(ui);
        self.cstr().label(ui);
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        ui.horizontal(|ui| {
            if let Some(prefix) = prefix {
                prefix.cstr().label(ui);
            }
            let mut hsva = self.clone().into();
            let r = ui.color_edit_button_hsva(&mut hsva).changed();
            if r {
                *self = hsva.into();
            }
            r
        })
        .inner
    }
}
impl Show for Entity {
    fn show(&self, prefix: Option<&str>, _: &Context, ui: &mut Ui) {
        prefix.show(ui);
        self.to_string().label(ui);
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        self.show(prefix, &Context::default(), ui);
        false
    }
}

impl Show for Expression {
    fn show(&self, prefix: Option<&str>, context: &Context, ui: &mut Ui) {
        prefix.show(ui);
        let l = self.cstr().as_label(ui).selectable(true);
        let (pos, galley, response) = l.layout_in_ui(ui);
        let mut text_shape = TextShape::new(pos, galley, MISSING_COLOR);
        let color = if response.hovered() {
            text_shape.override_text_color = Some(VISIBLE_BRIGHT);
            VISIBLE_BRIGHT
        } else {
            VISIBLE_DARK
        };
        ui.painter().add(Shape::Text(text_shape));
        response.on_hover_ui(|ui| match self.get_value(context) {
            Ok(v) => {
                v.show(None, context, ui);
            }
            Err(e) => {
                e.cstr().label(ui);
            }
        });
        let inner = <Self as Injector<Self>>::get_inner(self);
        if !inner.is_empty() {
            Frame::none()
                .inner_margin(Margin::symmetric(4.0, 4.0))
                .stroke(Stroke::new(1.0, color))
                .rounding(ROUNDING)
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        for i in inner {
                            i.show(None, context, ui);
                        }
                    });
                });
        }
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        let header_enabled = match self {
            Expression::Var(_)
            | Expression::V(_)
            | Expression::S(_)
            | Expression::F(_)
            | Expression::I(_)
            | Expression::B(_)
            | Expression::V2(_, _)
            | Expression::C(_) => true,
            Expression::One
            | Expression::Zero
            | Expression::GT
            | Expression::Owner
            | Expression::Target
            | Expression::UnitSize
            | Expression::Sin(..)
            | Expression::Cos(..)
            | Expression::Even(..)
            | Expression::Abs(..)
            | Expression::Floor(..)
            | Expression::Ceil(..)
            | Expression::Fract(..)
            | Expression::Sqr(..)
            | Expression::V2EE(..)
            | Expression::Macro(..)
            | Expression::Sum(..)
            | Expression::Sub(..)
            | Expression::Mul(..)
            | Expression::Div(..)
            | Expression::Max(..)
            | Expression::Min(..)
            | Expression::Mod(..)
            | Expression::And(..)
            | Expression::Or(..)
            | Expression::Equals(..)
            | Expression::GreaterThen(..)
            | Expression::LessThen(..)
            | Expression::If(..) => false,
        };
        let body_enabled = match self {
            Expression::One
            | Expression::Zero
            | Expression::GT
            | Expression::Owner
            | Expression::Target
            | Expression::UnitSize
            | Expression::Var(..)
            | Expression::V(..)
            | Expression::S(..)
            | Expression::F(..)
            | Expression::I(..)
            | Expression::B(..)
            | Expression::V2(..)
            | Expression::C(..) => false,
            Expression::Sin(..)
            | Expression::Cos(..)
            | Expression::Even(..)
            | Expression::Abs(..)
            | Expression::Floor(..)
            | Expression::Ceil(..)
            | Expression::Fract(..)
            | Expression::Sqr(..)
            | Expression::V2EE(..)
            | Expression::Macro(..)
            | Expression::Sum(..)
            | Expression::Sub(..)
            | Expression::Mul(..)
            | Expression::Div(..)
            | Expression::Max(..)
            | Expression::Min(..)
            | Expression::Mod(..)
            | Expression::And(..)
            | Expression::Or(..)
            | Expression::Equals(..)
            | Expression::GreaterThen(..)
            | Expression::LessThen(..)
            | Expression::If(..) => true,
        };
        let mut cf = CollapsingFrame::new_selector(self)
            .prefix(prefix)
            .wrapper(|d| d.wrap())
            .copy(|d| {
                copy_to_clipboard_op(d.get_data());
            })
            .paste(|d| match ClipboardPlugin::get() {
                Some(v) => d.inject_data(&v),
                None => error!("Clipboard is empty"),
            });
        if header_enabled {
            cf = cf.header(|d, ui| match d {
                Expression::Var(v) => v.show_mut(Some("x:"), ui),
                Expression::V(v) => v.show_mut(Some("x:"), ui),
                Expression::S(v) => v.show_mut(Some("x:"), ui),
                Expression::F(v) => v.show_mut(Some("x:"), ui),
                Expression::I(v) => v.show_mut(Some("x:"), ui),
                Expression::B(v) => v.show_mut(Some("x:"), ui),
                Expression::V2(x, y) => {
                    let x = x.show_mut(Some("x:"), ui);
                    y.show_mut(Some("y:"), ui) || x
                }
                Expression::C(v) => v.show_mut(Some("c:"), ui),
                _ => false,
            });
        }
        if body_enabled {
            cf = cf.body(|d, ui| match d {
                Expression::Sin(x)
                | Expression::Cos(x)
                | Expression::Even(x)
                | Expression::Abs(x)
                | Expression::Floor(x)
                | Expression::Ceil(x)
                | Expression::Fract(x)
                | Expression::Sqr(x) => x.show_mut(Some("x:"), ui),
                Expression::V2EE(a, b)
                | Expression::Macro(a, b)
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
                    let a = a.show_mut(Some("a:"), ui);
                    b.show_mut(Some("b:"), ui) || a
                }
                Expression::If(a, b, c) => {
                    let a = a.show_mut(Some("if:"), ui);
                    let b = b.show_mut(Some("then:"), ui);
                    c.show_mut(Some("else:"), ui) || a || b
                }
                _ => false,
            });
        }
        cf.ui(ui)
    }
}

impl Show for PainterAction {
    fn show(&self, prefix: Option<&str>, context: &Context, ui: &mut Ui) {
        ui.vertical(|ui| {
            let resp = ui
                .horizontal_wrapped(|ui| {
                    prefix.show(ui);
                    self.cstr().label(ui);
                    for i in <Self as Injector<Expression>>::get_inner(self) {
                        i.show(None, context, ui);
                    }
                })
                .inner;
            let inner = <Self as Injector<Self>>::get_inner(self);
            if !inner.is_empty() {
                Frame::none()
                    .inner_margin(Margin::same(8.0))
                    .rounding(ROUNDING)
                    .stroke(STROKE_DARK)
                    .show(ui, |ui| {
                        for i in inner {
                            i.show(None, context, ui);
                        }
                    });
            }
            resp
        })
        .inner
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
                ui.vertical(|ui| x.show_mut(Some("cnt:"), ui) | a.show_mut(Some("a:"), ui))
                    .inner
            }
            PainterAction::List(l) => l.show_mut(Some("list:"), ui),
            PainterAction::Paint => false,
        })
    }
}
impl Show for Material {
    fn show(&self, prefix: Option<&str>, context: &Context, ui: &mut Ui) {
        let r = prefix.show(ui);
        let rect = ui.available_rect_before_wrap();
        let mut p = Painter::new(rect, ui.ctx());
        for i in &self.0 {
            let _ = i.paint(context, &mut p, ui);
        }
        for i in &self.0 {
            i.show(None, context, ui);
        }
        r
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        self.0.show_mut(prefix, ui)
    }
}
impl Show for Trigger {
    fn show(&self, prefix: Option<&str>, _: &Context, ui: &mut Ui) {
        prefix.show(ui);
        self.cstr_cs(CYAN, CstrStyle::Bold).label(ui);
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        Selector::new(prefix.unwrap_or_default()).ui_enum(self, ui)
    }
}
