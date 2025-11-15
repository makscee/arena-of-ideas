use std::{hash::Hasher, mem};

use var_name::VarName;
use var_value::VarValue;

use super::*;

#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, EnumIter, AsRefStr, Display)]
pub enum Expression {
    #[default]
    one,
    zero,
    gt,
    owner,
    target,
    caster,
    attacker,
    unit_size,
    pi,
    pi2,
    x,

    all_units,
    all_enemy_units,
    all_ally_units,
    all_other_ally_units,
    adjacent_ally_units,
    adjacent_back,
    adjacent_front,

    var(VarName),
    var_or_zero(VarName),
    owner_var(VarName),
    target_var(VarName),
    caster_var(VarName),
    status_var(VarName),
    value(VarValue),

    string(String),
    f32(f32),
    f32_slider(f32),
    i32(i32),
    bool(bool),
    vec2(f32, f32),
    color(HexColor),
    lua_i32(String),
    lua_f32(String),

    state_var(Box<Expression>, VarName),

    dbg(Box<Expression>),
    sin(Box<Expression>),
    cos(Box<Expression>),
    even(Box<Expression>),
    abs(Box<Expression>),
    floor(Box<Expression>),
    ceil(Box<Expression>),
    fract(Box<Expression>),
    sqr(Box<Expression>),
    unit_vec(Box<Expression>),
    rand(Box<Expression>),
    random_unit(Box<Expression>),
    neg(Box<Expression>),

    to_f32(Box<Expression>),

    vec2_ee(Box<Expression>, Box<Expression>),
    str_macro(Box<Expression>, Box<Expression>),
    sum(Box<Expression>, Box<Expression>),
    sub(Box<Expression>, Box<Expression>),
    mul(Box<Expression>, Box<Expression>),
    div(Box<Expression>, Box<Expression>),
    max(Box<Expression>, Box<Expression>),
    min(Box<Expression>, Box<Expression>),
    r#mod(Box<Expression>, Box<Expression>),
    and(Box<Expression>, Box<Expression>),
    or(Box<Expression>, Box<Expression>),
    equals(Box<Expression>, Box<Expression>),
    greater_then(Box<Expression>, Box<Expression>),
    less_then(Box<Expression>, Box<Expression>),
    fallback(Box<Expression>, Box<Expression>),

    r#if(Box<Expression>, Box<Expression>, Box<Expression>),
    oklch(Box<Expression>, Box<Expression>, Box<Expression>),
}

impl std::hash::Hash for Expression {
    fn hash<H: Hasher>(&self, state: &mut H) {
        mem::discriminant(self).hash(state);
        match self {
            Expression::one
            | Expression::zero
            | Expression::pi
            | Expression::pi2
            | Expression::x
            | Expression::gt
            | Expression::unit_size
            | Expression::all_units
            | Expression::all_ally_units
            | Expression::all_other_ally_units
            | Expression::all_enemy_units
            | Expression::adjacent_ally_units
            | Expression::adjacent_back
            | Expression::adjacent_front
            | Expression::owner
            | Expression::attacker
            | Expression::caster
            | Expression::target => {}
            Expression::var(v)
            | Expression::target_var(v)
            | Expression::owner_var(v)
            | Expression::caster_var(v)
            | Expression::status_var(v)
            | Expression::var_or_zero(v) => v.hash(state),
            Expression::value(v) => v.hash(state),
            Expression::string(v) => v.hash(state),
            Expression::color(v) => v.hash(state),
            Expression::f32(v) => v.to_bits().hash(state),
            Expression::f32_slider(v) => v.to_bits().hash(state),
            Expression::i32(v) => v.hash(state),
            Expression::bool(v) => v.hash(state),
            Expression::vec2(x, y) => {
                x.to_bits().hash(state);
                y.to_bits().hash(state);
            }
            Expression::lua_i32(code) => code.hash(state),
            Expression::lua_f32(code) => code.hash(state),

            Expression::state_var(e, v) => {
                e.hash(state);
                v.hash(state);
            }

            Expression::sin(e)
            | Expression::cos(e)
            | Expression::sqr(e)
            | Expression::unit_vec(e)
            | Expression::rand(e)
            | Expression::random_unit(e)
            | Expression::even(e)
            | Expression::abs(e)
            | Expression::floor(e)
            | Expression::ceil(e)
            | Expression::to_f32(e)
            | Expression::fract(e)
            | Expression::dbg(e)
            | Expression::neg(e) => e.hash(state),
            Expression::str_macro(a, b)
            | Expression::vec2_ee(a, b)
            | Expression::sum(a, b)
            | Expression::sub(a, b)
            | Expression::mul(a, b)
            | Expression::div(a, b)
            | Expression::max(a, b)
            | Expression::min(a, b)
            | Expression::r#mod(a, b)
            | Expression::and(a, b)
            | Expression::or(a, b)
            | Expression::equals(a, b)
            | Expression::greater_then(a, b)
            | Expression::less_then(a, b)
            | Expression::fallback(a, b) => {
                a.hash(state);
                b.hash(state);
            }
            Expression::oklch(a, b, c) | Expression::r#if(a, b, c) => {
                a.hash(state);
                b.hash(state);
                c.hash(state);
            }
        }
    }
}
