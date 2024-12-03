use std::{hash::Hasher, mem};

use super::*;

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, EnumIter, AsRefStr)]
pub enum Expression {
    #[default]
    One,
    Zero,

    OppositeFaction,
    SlotPosition,
    GT,
    Beat,
    PI,
    PI2,
    Age,
    Index,
    T,

    Owner,
    Caster,
    Target,
    Status,

    AllAllyUnits,
    AllEnemyUnits,
    AllUnits,
    AllOtherUnits,
    AdjacentUnits,

    FilterStatusUnits(String, Box<Expression>),
    FilterNoStatusUnits(String, Box<Expression>),
    StatusEntity(String, Box<Expression>),

    Value(VarValue),
    Context(VarName),
    OwnerState(VarName),
    TargetState(VarName),
    CasterState(VarName),
    StatusState(String, VarName),
    OwnerStateLast(VarName),
    TargetStateLast(VarName),
    CasterStateLast(VarName),
    StatusStateLast(String, VarName),
    AbilityContext(String, VarName),
    AbilityState(String, VarName),
    StatusCharges(String),
    HexColor(String),
    F(f32),
    I(i32),
    B(bool),
    S(String),
    V2(f32, f32),

    Dbg(Box<Expression>),
    Ctx(Box<Expression>),
    ToI(Box<Expression>),
    ToF(Box<Expression>),
    Vec2E(Box<Expression>),
    UnitVec(Box<Expression>),
    VX(Box<Expression>),
    VY(Box<Expression>),
    Sin(Box<Expression>),
    Cos(Box<Expression>),
    Sqr(Box<Expression>),
    Even(Box<Expression>),
    Abs(Box<Expression>),
    Floor(Box<Expression>),
    Ceil(Box<Expression>),
    Fract(Box<Expression>),
    SlotUnit(Box<Expression>),
    RandomF(Box<Expression>),
    RandomUnit(Box<Expression>),
    ListCount(Box<Expression>),

    MaxUnit(Box<Expression>, Box<Expression>),
    RandomUnitSubset(Box<Expression>, Box<Expression>),
    Vec2EE(Box<Expression>, Box<Expression>),
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

    WithVar(VarName, Box<Expression>, Box<Expression>),
}

impl std::hash::Hash for Expression {
    fn hash<H: Hasher>(&self, state: &mut H) {
        mem::discriminant(self).hash(state);
        match self {
            Expression::One
            | Expression::Zero
            | Expression::OppositeFaction
            | Expression::SlotPosition
            | Expression::GT
            | Expression::Beat
            | Expression::PI
            | Expression::PI2
            | Expression::Age
            | Expression::Index
            | Expression::T
            | Expression::Owner
            | Expression::Caster
            | Expression::Target
            | Expression::Status
            | Expression::AllAllyUnits
            | Expression::AllEnemyUnits
            | Expression::AllUnits
            | Expression::AllOtherUnits
            | Expression::AdjacentUnits => {}
            Expression::FilterStatusUnits(s, e)
            | Expression::FilterNoStatusUnits(s, e)
            | Expression::StatusEntity(s, e) => {
                s.hash(state);
                e.hash(state);
            }
            Expression::Value(v) => v.hash(state),
            Expression::Context(v)
            | Expression::OwnerState(v)
            | Expression::TargetState(v)
            | Expression::CasterState(v)
            | Expression::OwnerStateLast(v)
            | Expression::TargetStateLast(v)
            | Expression::CasterStateLast(v) => v.hash(state),
            Expression::StatusState(s, v)
            | Expression::StatusStateLast(s, v)
            | Expression::AbilityContext(s, v)
            | Expression::AbilityState(s, v) => {
                s.hash(state);
                v.hash(state);
            }
            Expression::StatusCharges(s) | Expression::HexColor(s) => s.hash(state),
            Expression::F(v) => v.to_bits().hash(state),
            Expression::I(v) => v.hash(state),
            Expression::B(v) => v.hash(state),
            Expression::S(v) => v.hash(state),
            Expression::V2(x, y) => {
                x.to_bits().hash(state);
                y.to_bits().hash(state);
            }
            Expression::Dbg(e)
            | Expression::Ctx(e)
            | Expression::ToI(e)
            | Expression::ToF(e)
            | Expression::Vec2E(e)
            | Expression::UnitVec(e)
            | Expression::VX(e)
            | Expression::VY(e)
            | Expression::Sin(e)
            | Expression::Cos(e)
            | Expression::Sqr(e)
            | Expression::Even(e)
            | Expression::Abs(e)
            | Expression::Floor(e)
            | Expression::Ceil(e)
            | Expression::Fract(e)
            | Expression::SlotUnit(e)
            | Expression::RandomF(e)
            | Expression::RandomUnit(e)
            | Expression::ListCount(e) => e.hash(state),
            Expression::MaxUnit(a, b)
            | Expression::RandomUnitSubset(a, b)
            | Expression::Vec2EE(a, b)
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
            Expression::WithVar(v, a, b) => {
                v.hash(state);
                a.hash(state);
                b.hash(state);
            }
        }
    }
}
