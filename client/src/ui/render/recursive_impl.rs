use super::*;
use crate::ui::render::composers::recursive::{
    RecursiveField, RecursiveFieldMut, RecursiveValue, RecursiveValueMut,
};

// Expression
impl FRecursive for Expression {
    fn get_inner_fields(&self) -> Vec<RecursiveField<'_>> {
        match self {
            Expression::one
            | Expression::zero
            | Expression::gt
            | Expression::owner
            | Expression::target
            | Expression::attacker
            | Expression::caster
            | Expression::unit_size
            | Expression::pi
            | Expression::pi2
            | Expression::x
            | Expression::all_units
            | Expression::all_enemy_units
            | Expression::all_ally_units
            | Expression::all_other_ally_units
            | Expression::adjacent_ally_units
            | Expression::adjacent_back
            | Expression::adjacent_front => vec![],

            Expression::var(var)
            | Expression::target_var(var)
            | Expression::owner_var(var)
            | Expression::caster_var(var)
            | Expression::status_var(var)
            | Expression::var_or_zero(var) => {
                vec![RecursiveField::named("var", RecursiveValue::Var(var))]
            }
            Expression::value(val) => vec![RecursiveField::named(
                "value",
                RecursiveValue::VarValue(val),
            )],
            Expression::string(s) => {
                vec![RecursiveField::named("string", RecursiveValue::String(s))]
            }
            Expression::f32(f) => vec![RecursiveField::named("f32", RecursiveValue::F32(f))],
            Expression::f32_slider(f) => {
                vec![RecursiveField::named("f32_slider", RecursiveValue::F32(f))]
            }
            Expression::i32(i) => vec![RecursiveField::named("i32", RecursiveValue::I32(i))],
            Expression::bool(b) => vec![RecursiveField::named("bool", RecursiveValue::Bool(b))],
            Expression::color(c) => {
                vec![RecursiveField::named("color", RecursiveValue::HexColor(c))]
            }
            Expression::lua_i32(i) => {
                vec![RecursiveField::named("lua_i32", RecursiveValue::String(i))]
            }
            Expression::lua_f32(f) => {
                vec![RecursiveField::named("lua_f32", RecursiveValue::String(f))]
            }

            Expression::vec2(x, y) => vec![
                RecursiveField::named("x", RecursiveValue::F32(x)),
                RecursiveField::named("y", RecursiveValue::F32(y)),
            ],

            Expression::sin(expr)
            | Expression::cos(expr)
            | Expression::even(expr)
            | Expression::abs(expr)
            | Expression::floor(expr)
            | Expression::ceil(expr)
            | Expression::fract(expr)
            | Expression::dbg(expr)
            | Expression::sqr(expr)
            | Expression::unit_vec(expr)
            | Expression::rand(expr)
            | Expression::random_unit(expr)
            | Expression::neg(expr)
            | Expression::to_f32(expr) => {
                vec![RecursiveField::named(
                    "expr",
                    RecursiveValue::Expr(expr.as_ref()),
                )]
            }

            Expression::state_var(expr, var) => vec![
                RecursiveField::named("expr", RecursiveValue::Expr(expr.as_ref())),
                RecursiveField::named("var", RecursiveValue::Var(var)),
            ],

            Expression::vec2_ee(a, b) => vec![
                RecursiveField::named("x", RecursiveValue::Expr(a.as_ref())),
                RecursiveField::named("y", RecursiveValue::Expr(b.as_ref())),
            ],
            Expression::str_macro(template, value) => vec![
                RecursiveField::named("template", RecursiveValue::Expr(template.as_ref())),
                RecursiveField::named("value", RecursiveValue::Expr(value.as_ref())),
            ],
            Expression::sum(left, right)
            | Expression::sub(left, right)
            | Expression::mul(left, right)
            | Expression::div(left, right)
            | Expression::max(left, right)
            | Expression::min(left, right)
            | Expression::r#mod(left, right)
            | Expression::and(left, right)
            | Expression::or(left, right)
            | Expression::equals(left, right)
            | Expression::greater_then(left, right)
            | Expression::less_then(left, right) => vec![
                RecursiveField::named("left", RecursiveValue::Expr(left.as_ref())),
                RecursiveField::named("right", RecursiveValue::Expr(right.as_ref())),
            ],
            Expression::fallback(primary, fallback) => vec![
                RecursiveField::named("primary", RecursiveValue::Expr(primary.as_ref())),
                RecursiveField::named("fallback", RecursiveValue::Expr(fallback.as_ref())),
            ],

            Expression::r#if(condition, then_expr, else_expr) => vec![
                RecursiveField::named("condition", RecursiveValue::Expr(condition.as_ref())),
                RecursiveField::named("then", RecursiveValue::Expr(then_expr.as_ref())),
                RecursiveField::named("else", RecursiveValue::Expr(else_expr.as_ref())),
            ],
            Expression::oklch(l, c, h) => vec![
                RecursiveField::named("lightness", RecursiveValue::Expr(l.as_ref())),
                RecursiveField::named("chroma", RecursiveValue::Expr(c.as_ref())),
                RecursiveField::named("hue", RecursiveValue::Expr(h.as_ref())),
            ],
            Expression::list(exprs) => exprs
                .iter()
                .enumerate()
                .map(|(i, expr)| RecursiveField::indexed(i, RecursiveValue::Expr(expr)))
                .collect(),
        }
    }

    fn to_recursive_value(&self) -> RecursiveValue<'_> {
        RecursiveValue::Expr(self)
    }

    fn to_recursive_value_mut(&mut self) -> RecursiveValueMut<'_> {
        RecursiveValueMut::Expr(self)
    }

    fn get_inner_fields_mut(&mut self) -> Vec<RecursiveFieldMut<'_>> {
        match self {
            Expression::one
            | Expression::zero
            | Expression::gt
            | Expression::owner
            | Expression::target
            | Expression::attacker
            | Expression::caster
            | Expression::unit_size
            | Expression::pi
            | Expression::pi2
            | Expression::x
            | Expression::all_units
            | Expression::all_enemy_units
            | Expression::all_ally_units
            | Expression::all_other_ally_units
            | Expression::adjacent_ally_units
            | Expression::adjacent_back
            | Expression::adjacent_front => vec![],

            Expression::var(var)
            | Expression::owner_var(var)
            | Expression::target_var(var)
            | Expression::caster_var(var)
            | Expression::status_var(var)
            | Expression::var_or_zero(var) => {
                vec![RecursiveFieldMut::named("var", RecursiveValueMut::Var(var))]
            }
            Expression::value(val) => vec![RecursiveFieldMut::named(
                "value",
                RecursiveValueMut::VarValue(val),
            )],
            Expression::string(s) => {
                vec![RecursiveFieldMut::named(
                    "string",
                    RecursiveValueMut::String(s),
                )]
            }
            Expression::f32(f) => vec![RecursiveFieldMut::named("f32", RecursiveValueMut::F32(f))],
            Expression::f32_slider(f) => {
                vec![RecursiveFieldMut::named(
                    "f32_slider",
                    RecursiveValueMut::F32(f),
                )]
            }
            Expression::i32(i) => vec![RecursiveFieldMut::named("i32", RecursiveValueMut::I32(i))],
            Expression::bool(b) => {
                vec![RecursiveFieldMut::named("bool", RecursiveValueMut::Bool(b))]
            }
            Expression::color(c) => {
                vec![RecursiveFieldMut::named(
                    "color",
                    RecursiveValueMut::HexColor(c),
                )]
            }
            Expression::lua_i32(i) => {
                vec![RecursiveFieldMut::named(
                    "lua_i32",
                    RecursiveValueMut::String(i),
                )]
            }
            Expression::lua_f32(f) => {
                vec![RecursiveFieldMut::named(
                    "lua_f32",
                    RecursiveValueMut::String(f),
                )]
            }

            Expression::vec2(x, y) => vec![
                RecursiveFieldMut::named("x", RecursiveValueMut::F32(x)),
                RecursiveFieldMut::named("y", RecursiveValueMut::F32(y)),
            ],

            Expression::sin(expr)
            | Expression::cos(expr)
            | Expression::even(expr)
            | Expression::abs(expr)
            | Expression::floor(expr)
            | Expression::ceil(expr)
            | Expression::fract(expr)
            | Expression::dbg(expr)
            | Expression::sqr(expr)
            | Expression::unit_vec(expr)
            | Expression::rand(expr)
            | Expression::random_unit(expr)
            | Expression::neg(expr)
            | Expression::to_f32(expr) => {
                vec![RecursiveFieldMut::named(
                    "expr",
                    RecursiveValueMut::Expr(expr.as_mut()),
                )]
            }

            Expression::state_var(expr, var) => vec![
                RecursiveFieldMut::named("expr", RecursiveValueMut::Expr(expr.as_mut())),
                RecursiveFieldMut::named("var", RecursiveValueMut::Var(var)),
            ],

            Expression::vec2_ee(a, b) => vec![
                RecursiveFieldMut::named("x", RecursiveValueMut::Expr(a.as_mut())),
                RecursiveFieldMut::named("y", RecursiveValueMut::Expr(b.as_mut())),
            ],
            Expression::str_macro(template, value) => vec![
                RecursiveFieldMut::named("template", RecursiveValueMut::Expr(template.as_mut())),
                RecursiveFieldMut::named("value", RecursiveValueMut::Expr(value.as_mut())),
            ],
            Expression::sum(left, right)
            | Expression::sub(left, right)
            | Expression::mul(left, right)
            | Expression::div(left, right)
            | Expression::max(left, right)
            | Expression::min(left, right)
            | Expression::r#mod(left, right)
            | Expression::and(left, right)
            | Expression::or(left, right)
            | Expression::equals(left, right)
            | Expression::greater_then(left, right)
            | Expression::less_then(left, right) => vec![
                RecursiveFieldMut::named("left", RecursiveValueMut::Expr(left.as_mut())),
                RecursiveFieldMut::named("right", RecursiveValueMut::Expr(right.as_mut())),
            ],
            Expression::fallback(primary, fallback) => vec![
                RecursiveFieldMut::named("primary", RecursiveValueMut::Expr(primary.as_mut())),
                RecursiveFieldMut::named("fallback", RecursiveValueMut::Expr(fallback.as_mut())),
            ],

            Expression::r#if(condition, then_expr, else_expr) => vec![
                RecursiveFieldMut::named("condition", RecursiveValueMut::Expr(condition.as_mut())),
                RecursiveFieldMut::named("then", RecursiveValueMut::Expr(then_expr.as_mut())),
                RecursiveFieldMut::named("else", RecursiveValueMut::Expr(else_expr.as_mut())),
            ],
            Expression::oklch(l, c, h) => vec![
                RecursiveFieldMut::named("lightness", RecursiveValueMut::Expr(l.as_mut())),
                RecursiveFieldMut::named("chroma", RecursiveValueMut::Expr(c.as_mut())),
                RecursiveFieldMut::named("hue", RecursiveValueMut::Expr(h.as_mut())),
            ],
            Expression::list(exprs) => exprs
                .iter_mut()
                .enumerate()
                .map(|(i, expr)| RecursiveFieldMut::indexed(i, RecursiveValueMut::Expr(expr)))
                .collect(),
        }
    }
}

// Action
impl FRecursive for Action {
    fn get_inner_fields(&self) -> Vec<RecursiveField<'_>> {
        match self {
            Action::noop
            | Action::deal_damage
            | Action::heal_damage
            | Action::use_ability(..)
            | Action::apply_status(..) => vec![],

            Action::debug(expr)
            | Action::set_value(expr)
            | Action::add_value(expr)
            | Action::subtract_value(expr)
            | Action::add_target(expr)
            | Action::set_target(expr)
            | Action::set_status(expr)
            | Action::change_status_stax(expr) => {
                vec![RecursiveField::named(
                    "expr",
                    RecursiveValue::Expr(expr.as_ref()),
                )]
            }

            Action::repeat(count_expr, actions) => {
                let mut fields = vec![RecursiveField::named(
                    "count",
                    RecursiveValue::Expr(count_expr.as_ref()),
                )];
                for (i, action) in actions.iter().enumerate() {
                    fields.push(RecursiveField::indexed(
                        i,
                        RecursiveValue::Action(action.as_ref()),
                    ));
                }
                fields
            }
        }
    }

    fn to_recursive_value(&self) -> RecursiveValue<'_> {
        RecursiveValue::Action(self)
    }

    fn to_recursive_value_mut(&mut self) -> RecursiveValueMut<'_> {
        RecursiveValueMut::Action(self)
    }

    fn get_inner_fields_mut(&mut self) -> Vec<RecursiveFieldMut<'_>> {
        match self {
            Action::noop
            | Action::deal_damage
            | Action::heal_damage
            | Action::use_ability(..)
            | Action::apply_status(..) => vec![],

            Action::debug(expr)
            | Action::set_value(expr)
            | Action::add_value(expr)
            | Action::subtract_value(expr)
            | Action::set_target(expr)
            | Action::add_target(expr)
            | Action::set_status(expr)
            | Action::change_status_stax(expr) => {
                vec![RecursiveFieldMut::named(
                    "expr",
                    RecursiveValueMut::Expr(expr.as_mut()),
                )]
            }

            Action::repeat(count_expr, actions) => {
                let mut fields = vec![RecursiveFieldMut::named(
                    "count",
                    RecursiveValueMut::Expr(count_expr.as_mut()),
                )];
                for (i, action) in actions.iter_mut().enumerate() {
                    fields.push(RecursiveFieldMut::indexed(
                        i,
                        RecursiveValueMut::Action(action.as_mut()),
                    ));
                }
                fields
            }
        }
    }
}

// Material
impl FRecursive for Material {
    fn get_inner_fields(&self) -> Vec<RecursiveField<'_>> {
        vec![RecursiveField::named(
            "script",
            RecursiveValue::String(&self.0.code),
        )]
    }

    fn to_recursive_value(&self) -> RecursiveValue<'_> {
        RecursiveValue::Material(self)
    }

    fn to_recursive_value_mut(&mut self) -> RecursiveValueMut<'_> {
        RecursiveValueMut::Material(self)
    }

    fn get_inner_fields_mut(&mut self) -> Vec<RecursiveFieldMut<'_>> {
        vec![RecursiveFieldMut::named(
            "script",
            RecursiveValueMut::String(&mut self.0.code),
        )]
    }
}

// Reaction
impl FRecursive for Behavior {
    fn get_inner_fields(&self) -> Vec<RecursiveField<'_>> {
        self.effect
            .actions
            .iter()
            .enumerate()
            .map(|(i, action)| RecursiveField::indexed(i, RecursiveValue::Action(action)))
            .collect()
    }

    fn to_recursive_value(&self) -> RecursiveValue<'_> {
        RecursiveValue::Behavior(self)
    }

    fn to_recursive_value_mut(&mut self) -> RecursiveValueMut<'_> {
        RecursiveValueMut::Behavior(self)
    }

    fn get_inner_fields_mut(&mut self) -> Vec<RecursiveFieldMut<'_>> {
        self.effect
            .actions
            .iter_mut()
            .enumerate()
            .map(|(i, action)| RecursiveFieldMut::indexed(i, RecursiveValueMut::Action(action)))
            .collect()
    }
}

// Primitive types have no inner fields
impl FRecursive for i32 {
    fn to_recursive_value(&self) -> RecursiveValue<'_> {
        RecursiveValue::I32(self)
    }

    fn to_recursive_value_mut(&mut self) -> RecursiveValueMut<'_> {
        RecursiveValueMut::I32(self)
    }
}

impl FRecursive for f32 {
    fn to_recursive_value(&self) -> RecursiveValue<'_> {
        RecursiveValue::F32(self)
    }

    fn to_recursive_value_mut(&mut self) -> RecursiveValueMut<'_> {
        RecursiveValueMut::F32(self)
    }
}

impl FRecursive for String {
    fn to_recursive_value(&self) -> RecursiveValue<'_> {
        RecursiveValue::String(self)
    }

    fn to_recursive_value_mut(&mut self) -> RecursiveValueMut<'_> {
        RecursiveValueMut::String(self)
    }
}

impl FRecursive for bool {
    fn to_recursive_value(&self) -> RecursiveValue<'_> {
        RecursiveValue::Bool(self)
    }

    fn to_recursive_value_mut(&mut self) -> RecursiveValueMut<'_> {
        RecursiveValueMut::Bool(self)
    }
}

impl FRecursive for VarName {
    fn to_recursive_value(&self) -> RecursiveValue<'_> {
        RecursiveValue::Var(self)
    }

    fn to_recursive_value_mut(&mut self) -> RecursiveValueMut<'_> {
        RecursiveValueMut::Var(self)
    }
}

impl FRecursive for VarValue {
    fn to_recursive_value(&self) -> RecursiveValue<'_> {
        RecursiveValue::VarValue(self)
    }

    fn to_recursive_value_mut(&mut self) -> RecursiveValueMut<'_> {
        RecursiveValueMut::VarValue(self)
    }
}

impl FRecursive for HexColor {
    fn to_recursive_value(&self) -> RecursiveValue<'_> {
        RecursiveValue::HexColor(self)
    }

    fn to_recursive_value_mut(&mut self) -> RecursiveValueMut<'_> {
        RecursiveValueMut::HexColor(self)
    }
}

impl FRecursive for Vec2 {
    fn to_recursive_value(&self) -> RecursiveValue<'_> {
        RecursiveValue::Vec2(self)
    }

    fn to_recursive_value_mut(&mut self) -> RecursiveValueMut<'_> {
        RecursiveValueMut::Vec2(self)
    }
}

// Box implementation
impl FRecursive for Box<Expression> {
    fn get_inner_fields(&self) -> Vec<RecursiveField<'_>> {
        self.as_ref().get_inner_fields()
    }

    fn get_inner_fields_mut(&mut self) -> Vec<RecursiveFieldMut<'_>> {
        self.as_mut().get_inner_fields_mut()
    }

    fn to_recursive_value(&self) -> RecursiveValue<'_> {
        self.as_ref().to_recursive_value()
    }

    fn to_recursive_value_mut(&mut self) -> RecursiveValueMut<'_> {
        self.as_mut().to_recursive_value_mut()
    }
}

// Vec implementation

// Option implementation
impl FRecursive for Option<Expression> {
    fn get_inner_fields(&self) -> Vec<RecursiveField<'_>> {
        match self {
            Some(expr) => vec![RecursiveField::named("value", RecursiveValue::Expr(expr))],
            None => vec![],
        }
    }

    fn get_inner_fields_mut(&mut self) -> Vec<RecursiveFieldMut<'_>> {
        match self {
            Some(expr) => vec![RecursiveFieldMut::named(
                "value",
                RecursiveValueMut::Expr(expr),
            )],
            None => vec![],
        }
    }

    fn to_recursive_value(&self) -> RecursiveValue<'_> {
        match self {
            Some(expr) => RecursiveValue::Expr(expr),
            None => panic!("Cannot convert None to RecursiveValue"),
        }
    }

    fn to_recursive_value_mut(&mut self) -> RecursiveValueMut<'_> {
        match self {
            Some(expr) => RecursiveValueMut::Expr(expr),
            None => panic!("Cannot convert None to RecursiveValueMut"),
        }
    }
}
