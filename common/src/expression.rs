use std::{fmt::Display, hash::Hasher, mem};

use bevy::math::vec2;
use bevy_egui::egui::Color32;

use super::*;

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, EnumIter, AsRefStr)]
pub enum Expression {
    #[default]
    One,
    Zero,
    GT,

    Var(VarName),
    V(VarValue),

    S(String),
    F(f32),
    I(i32),
    B(bool),
    V2(f32, f32),
    C(String),

    Sin(Box<Expression>),
    Cos(Box<Expression>),
    Even(Box<Expression>),
    Abs(Box<Expression>),
    Floor(Box<Expression>),
    Ceil(Box<Expression>),
    Fract(Box<Expression>),
    Sqr(Box<Expression>),

    Macro(Box<Expression>, Box<Expression>),
    Sum(Box<Expression>, Box<Expression>),
    Sub(Box<Expression>, Box<Expression>),
    Mul(Box<Expression>, Box<Expression>),
    Div(Box<Expression>, Box<Expression>),
    Max(Box<Expression>, Box<Expression>),
    Min(Box<Expression>, Box<Expression>),
    Mod(Box<Expression>, Box<Expression>),
    And(Box<Expression>, Box<Expression>),
    Or(Box<Expression>, Box<Expression>),
    Equals(Box<Expression>, Box<Expression>),
    GreaterThen(Box<Expression>, Box<Expression>),
    LessThen(Box<Expression>, Box<Expression>),

    If(Box<Expression>, Box<Expression>, Box<Expression>),
}

impl std::hash::Hash for Expression {
    fn hash<H: Hasher>(&self, state: &mut H) {
        mem::discriminant(self).hash(state);
        match self {
            Expression::One | Expression::Zero | Expression::GT => {}
            Expression::Var(v) => v.hash(state),
            Expression::V(v) => v.hash(state),
            Expression::S(v) | Expression::C(v) => v.hash(state),
            Expression::F(v) => v.to_bits().hash(state),
            Expression::I(v) => v.hash(state),
            Expression::B(v) => v.hash(state),
            Expression::V2(x, y) => {
                x.to_bits().hash(state);
                y.to_bits().hash(state);
            }

            Expression::Sin(e)
            | Expression::Cos(e)
            | Expression::Sqr(e)
            | Expression::Even(e)
            | Expression::Abs(e)
            | Expression::Floor(e)
            | Expression::Ceil(e)
            | Expression::Fract(e) => e.hash(state),
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
                a.hash(state);
                b.hash(state);
            }
            Expression::If(i, t, e) => {
                i.hash(state);
                t.hash(state);
                e.hash(state);
            }
        }
    }
}
impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.cstr_expanded())
    }
}
impl ToCstr for Expression {
    fn cstr(&self) -> Cstr {
        self.as_ref().cstr_c(YELLOW)
    }
    fn cstr_expanded(&self) -> Cstr {
        let inner = match self {
            Expression::One | Expression::Zero | Expression::GT => String::default(),
            Expression::Var(v) => v.cstr(),
            Expression::V(v) => v.cstr(),
            Expression::S(v) => v.to_owned(),
            Expression::F(v) => v.cstr(),
            Expression::I(v) => v.cstr(),
            Expression::B(v) => v.cstr(),
            Expression::V2(x, y) => vec2(*x, *y).cstr(),
            Expression::C(c) => match Color32::from_hex(c) {
                Ok(color) => c.cstr_c(color),
                Err(e) => format!("{c} [s {e:?}]",).cstr_c(RED),
            },
            Expression::Sin(x)
            | Expression::Cos(x)
            | Expression::Even(x)
            | Expression::Abs(x)
            | Expression::Floor(x)
            | Expression::Ceil(x)
            | Expression::Fract(x)
            | Expression::Sqr(x) => x.cstr(),
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
            | Expression::LessThen(a, b) => format!("{a}, {b}"),
            Expression::If(a, b, c) => format!("{a}, {b}, {c}"),
        };
        if inner.is_empty() {
            self.cstr()
        } else {
            format!("{}({inner})", self.cstr())
        }
    }
}

impl Show for Expression {
    fn show(&self, prefix: Option<&str>, ui: &mut Ui) {
        format!("{}{}", prefix.unwrap_or_default(), self.cstr_expanded()).label(ui);
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        ui.horizontal(|ui| {
            Selector::new(prefix.unwrap_or_default()).ui_enum(self, ui)
                || match self {
                    Expression::One | Expression::Zero | Expression::GT => false,
                    Expression::Var(v) => v.show_mut(None, ui),
                    Expression::V(v) => v.show_mut(None, ui),
                    Expression::S(v) => v.show_mut(None, ui),
                    Expression::F(v) => v.show_mut(None, ui),
                    Expression::I(v) => v.show_mut(None, ui),
                    Expression::B(v) => v.show_mut(None, ui),
                    Expression::C(v) => v.show_mut(None, ui),
                    Expression::V2(x, y) => {
                        let mut v = vec2(*x, *y);
                        if v.show_mut(None, ui) {
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
                    | Expression::Sqr(x) => x.show_mut(None, ui),
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
                        a.show_mut(Some("a:".into()), ui) || b.show_mut(Some("b:".into()), ui)
                    }
                    Expression::If(i, t, e) => {
                        i.show_mut(Some("if:".into()), ui)
                            || t.show_mut(Some("then:".into()), ui)
                            || e.show_mut(Some("else:".into()), ui)
                    }
                }
        })
        .inner
    }
}
