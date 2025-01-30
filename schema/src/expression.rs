use std::{hash::Hasher, mem};

use var_name::VarName;
use var_value::VarValue;

use super::*;

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, EnumIter, AsRefStr)]
pub enum Expression {
    #[default]
    One,
    Zero,
    GT,
    Owner,
    Target,
    UnitSize,
    PI,
    PI2,

    AllUnits,

    Var(VarName),
    V(VarValue),

    S(String),
    F(f32),
    FSlider(f32),
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
    UnitVec(Box<Expression>),
    Rand(Box<Expression>),
    RandomUnit(Box<Expression>),

    ToF(Box<Expression>),

    V2EE(Box<Expression>, Box<Expression>),
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
    Oklch(Box<Expression>, Box<Expression>, Box<Expression>),
}

impl std::hash::Hash for Expression {
    fn hash<H: Hasher>(&self, state: &mut H) {
        mem::discriminant(self).hash(state);
        match self {
            Expression::One
            | Expression::Zero
            | Expression::PI
            | Expression::PI2
            | Expression::GT
            | Expression::UnitSize
            | Expression::AllUnits
            | Expression::Owner
            | Expression::Target => {}
            Expression::Var(v) => v.hash(state),
            Expression::V(v) => v.hash(state),
            Expression::S(v) | Expression::C(v) => v.hash(state),
            Expression::F(v) => v.to_bits().hash(state),
            Expression::FSlider(v) => v.to_bits().hash(state),
            Expression::I(v) => v.hash(state),
            Expression::B(v) => v.hash(state),
            Expression::V2(x, y) => {
                x.to_bits().hash(state);
                y.to_bits().hash(state);
            }

            Expression::Sin(e)
            | Expression::Cos(e)
            | Expression::Sqr(e)
            | Expression::UnitVec(e)
            | Expression::Rand(e)
            | Expression::RandomUnit(e)
            | Expression::Even(e)
            | Expression::Abs(e)
            | Expression::Floor(e)
            | Expression::Ceil(e)
            | Expression::ToF(e)
            | Expression::Fract(e) => e.hash(state),
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
                a.hash(state);
                b.hash(state);
            }
            Expression::Oklch(a, b, c) | Expression::If(a, b, c) => {
                a.hash(state);
                b.hash(state);
                c.hash(state);
            }
        }
    }
}
