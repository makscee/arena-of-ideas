use egui::Response;
use serde::de::DeserializeOwned;

use super::*;

fn context_menu(d: &impl StringData, r: &Response, ui: &mut Ui) {
    r.context_menu(|ui| {
        if "Copy".cstr().button(ui).clicked() {
            let s = d.get_data();
            OperationsPlugin::add(move |world| {
                copy_to_clipboard(&s, world);
            });
            ui.close_menu();
        }
        if "Close".cstr_c(VISIBLE_DARK).button(ui).clicked() {
            ui.close_menu();
        }
    });
}
pub trait ShowPrefix {
    fn show(&self, ui: &mut Ui) -> Response;
}
impl ShowPrefix for Option<&str> {
    fn show(&self, ui: &mut Ui) -> Response {
        if let Some(s) = self {
            s.cstr_cs(VISIBLE_DARK, CstrStyle::Small).label(ui)
        } else {
            empty_response(ui.ctx().clone())
        }
    }
}
pub trait Show: StringData {
    fn show(&self, prefix: Option<&str>, context: &Context, ui: &mut Ui) -> Response {
        let r = self.show_self(prefix, context, ui);
        context_menu(self, &r, ui);
        r
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> Response {
        let r = self.show_self_mut(prefix, ui);
        context_menu(self, &r, ui);
        r
    }
    fn show_self(&self, prefix: Option<&str>, context: &Context, ui: &mut Ui) -> Response;
    fn show_self_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> Response;
}

impl<T> Show for Vec<Box<T>>
where
    T: Show + Default + Serialize + DeserializeOwned,
{
    fn show_self(&self, prefix: Option<&str>, context: &Context, ui: &mut Ui) -> Response {
        let mut r = prefix.show(ui);
        for (i, v) in self.into_iter().enumerate() {
            r = r.union(v.show(Some(&format!("[vd {i}:]")), context, ui));
        }
        r
    }
    fn show_self_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> Response {
        let mut r = prefix.show(ui);
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
                        if i < len && ">".cstr_s(CstrStyle::Bold).button(ui).clicked() {
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
                r.changed |= a.show_mut(Some(&i.to_string()), ui).changed();
            });
        }
        if self.is_empty() && plus_btn(ui) {
            insert = Some(0);
        }
        if let Some(delete) = delete {
            self.remove(delete);
        }
        if let Some(index) = insert {
            self.insert(index, default());
        }
        if let Some((a, b)) = swap {
            self.swap(a, b);
        }
        r
    }
}
impl Show for VarName {
    fn show_self(&self, prefix: Option<&str>, _: &Context, ui: &mut Ui) -> Response {
        prefix.show(ui) | self.cstr_expanded().label(ui)
    }
    fn show_self_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> Response {
        Selector::new(prefix.unwrap_or_default()).ui_enum(self, ui)
    }
}
impl Show for VarValue {
    fn show_self(&self, prefix: Option<&str>, context: &Context, ui: &mut Ui) -> Response {
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
    fn show_self_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> Response {
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
    fn show_self(&self, prefix: Option<&str>, _: &Context, ui: &mut Ui) -> Response {
        prefix.show(ui) | self.cstr().label(ui)
    }
    fn show_self_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            if let Some(prefix) = prefix {
                prefix.cstr().label(ui);
            }
            DragValue::new(self).ui(ui)
        })
        .inner
    }
}
impl Show for f32 {
    fn show_self(&self, prefix: Option<&str>, _: &Context, ui: &mut Ui) -> Response {
        prefix.show(ui) | self.cstr().label(ui)
    }
    fn show_self_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            if let Some(prefix) = prefix {
                prefix.cstr().label(ui);
            }
            DragValue::new(self).ui(ui)
        })
        .inner
    }
}
impl Show for u64 {
    fn show_self(&self, prefix: Option<&str>, _: &Context, ui: &mut Ui) -> Response {
        prefix.show(ui) | self.cstr().label(ui)
    }
    fn show_self_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            if let Some(prefix) = prefix {
                prefix.cstr().label(ui);
            }
            DragValue::new(self).ui(ui)
        })
        .inner
    }
}
impl Show for bool {
    fn show_self(&self, prefix: Option<&str>, _: &Context, ui: &mut Ui) -> Response {
        prefix.show(ui) | self.cstr().label(ui)
    }
    fn show_self_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            Checkbox::new(self, prefix.unwrap_or_default().to_owned().widget(1.0, ui)).ui(ui)
        })
        .inner
    }
}
impl Show for Vec2 {
    fn show_self(&self, prefix: Option<&str>, _: &Context, ui: &mut Ui) -> Response {
        prefix.show(ui) | self.cstr().label(ui)
    }
    fn show_self_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            if let Some(prefix) = prefix {
                prefix.cstr().label(ui);
            }
            let rx = DragValue::new(&mut self.x).prefix("x:").ui(ui);
            let ry = DragValue::new(&mut self.y).prefix("y:").ui(ui);
            rx.union(ry)
        })
        .inner
    }
}
impl Show for String {
    fn show_self(&self, prefix: Option<&str>, _: &Context, ui: &mut Ui) -> Response {
        prefix.show(ui) | self.cstr().label(ui)
    }
    fn show_self_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> Response {
        Input::new(prefix.unwrap_or_default()).ui_string(self, ui)
    }
}
impl Show for Color {
    fn show_self(&self, prefix: Option<&str>, _: &Context, ui: &mut Ui) -> Response {
        prefix.show(ui) | self.cstr().label(ui)
    }
    fn show_self_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            let mut c = self.c32();
            let r = c.show_mut(prefix, ui);
            if r.changed() {
                *self = c.to_color();
            }
            r
        })
        .inner
    }
}
impl Show for Color32 {
    fn show_self(&self, prefix: Option<&str>, _: &Context, ui: &mut Ui) -> Response {
        prefix.show(ui) | self.cstr().label(ui)
    }
    fn show_self_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            if let Some(prefix) = prefix {
                prefix.cstr().label(ui);
            }
            let mut hsva = self.clone().into();
            let r = ui.color_edit_button_hsva(&mut hsva);
            if r.changed() {
                *self = hsva.into();
            }
            r
        })
        .inner
    }
}
impl Show for Entity {
    fn show_self(&self, prefix: Option<&str>, _: &Context, ui: &mut Ui) -> Response {
        prefix.show(ui) | self.to_string().label(ui)
    }
    fn show_self_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> Response {
        self.show(prefix, &Context::default(), ui)
    }
}

impl Show for Expression {
    fn show_self(&self, prefix: Option<&str>, context: &Context, ui: &mut Ui) -> Response {
        let r = prefix.show(ui);
        let l = self.cstr().as_label(ui).selectable(true);
        let (pos, galley, response) = l.layout_in_ui(ui);
        let response = response | r;
        let mut text_shape = TextShape::new(pos, galley, MISSING_COLOR);
        let color = if response.hovered() {
            text_shape.override_text_color = Some(VISIBLE_BRIGHT);
            VISIBLE_BRIGHT
        } else {
            VISIBLE_DARK
        };
        ui.painter().add(Shape::Text(text_shape));
        let response = response.on_hover_ui(|ui| match self.get_value(context) {
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
        response
    }
    fn show_self_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> Response {
        CollapsingSelector::ui(self, prefix, ui, |v, ui| match v {
            Expression::One
            | Expression::Zero
            | Expression::GT
            | Expression::UnitSize
            | Expression::Owner
            | Expression::Target => false,
            Expression::Var(v) => v.show_mut(Some("v:"), ui).changed(),
            Expression::V(v) => v.show_mut(Some("v:"), ui).changed(),
            Expression::S(v) => v.show_mut(Some("v:"), ui).changed(),
            Expression::F(v) => v.show_mut(Some("v:"), ui).changed(),
            Expression::I(v) => v.show_mut(Some("v:"), ui).changed(),
            Expression::B(v) => v.show_mut(Some("v:"), ui).changed(),
            Expression::C(v) => v.show_mut(Some("v:"), ui).changed(),
            Expression::V2(x, y) => {
                let mut v = vec2(*x, *y);
                if v.show_mut(Some("v:"), ui).changed() {
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
            | Expression::Sqr(x) => x.show_mut(Some("x:"), ui).changed(),
            Expression::Macro(a, b)
            | Expression::V2EE(a, b)
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
                    r |= a.show_mut(Some("a:".into()), ui).changed();
                    r |= b.show_mut(Some("b:".into()), ui).changed();
                });
                r
            }
            Expression::If(i, t, e) => {
                let mut r = false;
                ui.vertical(|ui| {
                    r |= i.show_mut(Some("if:".into()), ui).changed();
                    r |= t.show_mut(Some("then:".into()), ui).changed();
                    r |= e.show_mut(Some("else:".into()), ui).changed();
                });
                r
            }
        })
    }
}

impl Show for PainterAction {
    fn show_self(&self, prefix: Option<&str>, context: &Context, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            let resp = ui
                .horizontal_wrapped(|ui| {
                    let mut r = prefix.show(ui);
                    r |= self.cstr().label(ui);
                    for i in <Self as Injector<Expression>>::get_inner(self) {
                        i.show(None, context, ui);
                    }
                    r
                })
                .inner;
            let inner = <Self as Injector<Self>>::get_inner(self);
            if !inner.is_empty() {
                Frame::none()
                    .inner_margin(Margin::same(8.0))
                    .rounding(ROUNDING)
                    .stroke(Stroke::new(
                        1.0,
                        if resp.hovered() {
                            VISIBLE_BRIGHT
                        } else {
                            VISIBLE_DARK
                        },
                    ))
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
    fn show_self_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> Response {
        CollapsingSelector::ui(self, prefix, ui, |v, ui| match v {
            PainterAction::Circle(x)
            | PainterAction::Rectangle(x)
            | PainterAction::Text(x)
            | PainterAction::Hollow(x)
            | PainterAction::Translate(x)
            | PainterAction::Rotate(x)
            | PainterAction::Scale(x)
            | PainterAction::Alpha(x)
            | PainterAction::Color(x) => x.show_mut(Some("x:"), ui).changed(),
            PainterAction::Repeat(x, a) => ui
                .vertical(|ui| x.show_mut(Some("cnt:"), ui) | a.show_mut(Some("a:"), ui))
                .inner
                .changed(),
            PainterAction::List(l) => l.show_mut(Some("list:"), ui).changed(),
            PainterAction::Paint => false,
        })
    }
}
impl Show for Material {
    fn show_self(&self, prefix: Option<&str>, context: &Context, ui: &mut Ui) -> Response {
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
    fn show_self_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> Response {
        self.0.show_mut(prefix, ui)
    }
}
impl Show for Trigger {
    fn show_self(&self, prefix: Option<&str>, _: &Context, ui: &mut Ui) -> Response {
        let mut r = prefix.show(ui);
        r | self.cstr_cs(CYAN, CstrStyle::Bold).label(ui)
    }
    fn show_self_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> Response {
        Selector::new(prefix.unwrap_or_default()).ui_enum(self, ui)
    }
}
