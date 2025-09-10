use super::*;

pub struct RecursiveField<'a> {
    pub name: String,
    pub value: RecursiveValue<'a>,
}

#[derive(Copy, Clone)]
pub enum RecursiveValue<'a> {
    Expr(&'a Expression),
    Action(&'a Action),
    PainterAction(&'a PainterAction),
    Var(&'a VarName),
    VarValue(&'a VarValue),
    HexColor(&'a HexColor),
    String(&'a String),
    I32(&'a i32),
    F32(&'a f32),
    Bool(&'a bool),
    Vec2(&'a Vec2),
    Reaction(&'a Reaction),
    Material(&'a Material),
}

#[derive(Debug)]
pub struct RecursiveFieldMut<'a> {
    pub name: String,
    pub value: RecursiveValueMut<'a>,
}

#[derive(Debug)]
pub enum RecursiveValueMut<'a> {
    Expr(&'a mut Expression),
    Action(&'a mut Action),
    PainterAction(&'a mut PainterAction),
    Var(&'a mut VarName),
    VarValue(&'a mut VarValue),
    HexColor(&'a mut HexColor),
    String(&'a mut String),
    I32(&'a mut i32),
    F32(&'a mut f32),
    Bool(&'a mut bool),
    Vec2(&'a mut Vec2),
    Reaction(&'a mut Reaction),
    Material(&'a mut Material),
}

impl<'a> RecursiveField<'a> {
    pub fn named(name: &str, value: RecursiveValue<'a>) -> Self {
        Self {
            name: name.to_string(),
            value,
        }
    }

    pub fn indexed(index: usize, value: RecursiveValue<'a>) -> Self {
        Self {
            name: index.to_string(),
            value,
        }
    }
}

impl<'a> RecursiveFieldMut<'a> {
    pub fn named(name: &str, value: RecursiveValueMut<'a>) -> Self {
        Self {
            name: name.to_string(),
            value,
        }
    }

    pub fn indexed(index: usize, value: RecursiveValueMut<'a>) -> Self {
        Self {
            name: index.to_string(),
            value,
        }
    }
}

#[macro_export]
macro_rules! call_on_recursive_value {
    ($field:expr, $func:ident $(, $arg:expr)*) => {
        match $field.value {
            RecursiveValue::Expr(v) => v.$func($($arg),*),
            RecursiveValue::Action(v) => v.$func($($arg),*),
            RecursiveValue::PainterAction(v) => v.$func($($arg),*),
            RecursiveValue::Var(v) => v.$func($($arg),*),
            RecursiveValue::VarValue(v) => v.$func($($arg),*),
            RecursiveValue::HexColor(v) => v.$func($($arg),*),
            RecursiveValue::String(v) => v.$func($($arg),*),
            RecursiveValue::I32(v) => v.$func($($arg),*),
            RecursiveValue::F32(v) => v.$func($($arg),*),
            RecursiveValue::Bool(v) => v.$func($($arg),*),
            RecursiveValue::Vec2(v) => v.$func($($arg),*),
            RecursiveValue::Reaction(v) => v.$func($($arg),*),
            RecursiveValue::Material(v) => v.$func($($arg),*),
        }
    };
}

#[macro_export]
macro_rules! call_pass_recursive_value {
    ($field:expr, $func:ident $(, $arg:expr)*) => {
        match $field.value {
            RecursiveValue::Expr(v) => $func(v, $($arg),*),
            RecursiveValue::Action(v) => $func(v, $($arg),*),
            RecursiveValue::PainterAction(v) => $func(v, $($arg),*),
            RecursiveValue::Var(v) => $func(v, $($arg),*),
            RecursiveValue::VarValue(v) => $func(v, $($arg),*),
            RecursiveValue::HexColor(v) => $func(v, $($arg),*),
            RecursiveValue::String(v) => $func(v, $($arg),*),
            RecursiveValue::I32(v) => $func(v, $($arg),*),
            RecursiveValue::F32(v) => $func(v, $($arg),*),
            RecursiveValue::Bool(v) => $func(v, $($arg),*),
            RecursiveValue::Vec2(v) => $func(v, $($arg),*),
            RecursiveValue::Reaction(v) => $func(v, $($arg),*),
            RecursiveValue::Material(v) => $func(v, $($arg),*),
        }
    };
}

#[macro_export]
macro_rules! call_on_recursive_value_mut {
    ($field:expr, $func:ident $(, $arg:expr)*) => {
        match &mut $field.value {
            RecursiveValueMut::Expr(v) => v.$func($($arg),*),
            RecursiveValueMut::Action(v) => v.$func($($arg),*),
            RecursiveValueMut::PainterAction(v) => v.$func($($arg),*),
            RecursiveValueMut::Var(v) => v.$func($($arg),*),
            RecursiveValueMut::VarValue(v) => v.$func($($arg),*),
            RecursiveValueMut::HexColor(v) => v.$func($($arg),*),
            RecursiveValueMut::String(v) => v.$func($($arg),*),
            RecursiveValueMut::I32(v) => v.$func($($arg),*),
            RecursiveValueMut::F32(v) => v.$func($($arg),*),
            RecursiveValueMut::Bool(v) => v.$func($($arg),*),
            RecursiveValueMut::Vec2(v) => v.$func($($arg),*),
            RecursiveValueMut::Reaction(v) => v.$func($($arg),*),
            RecursiveValueMut::Material(v) => v.$func($($arg),*),
        }
    };
}

#[macro_export]
macro_rules! call_pass_recursive_value_mut {
    ($field:expr, $func:ident $(, $arg:expr)*) => {
        match &mut $field.value {
            RecursiveValueMut::Expr(v) => $func(v, $($arg),*),
            RecursiveValueMut::Action(v) => $func(v, $($arg),*),
            RecursiveValueMut::PainterAction(v) => $func(v, $($arg),*),
            RecursiveValueMut::Var(v) => $func(v, $($arg),*),
            RecursiveValueMut::VarValue(v) => $func(v, $($arg),*),
            RecursiveValueMut::HexColor(v) => $func(v, $($arg),*),
            RecursiveValueMut::String(v) => $func(v, $($arg),*),
            RecursiveValueMut::I32(v) => $func(v, $($arg),*),
            RecursiveValueMut::F32(v) => $func(v, $($arg),*),
            RecursiveValueMut::Bool(v) => $func(v, $($arg),*),
            RecursiveValueMut::Vec2(v) => $func(v, $($arg),*),
            RecursiveValueMut::Reaction(v) => $func(v, $($arg),*),
            RecursiveValueMut::Material(v) => $func(v, $($arg),*),
        }
    };
}

impl<'a> RecursiveValueMut<'a> {
    /// Replace expression and move matching fields
    pub fn replace_expr_and_move_fields(expr: &mut Expression, mut new_expr: Expression) {
        std::mem::swap(expr, &mut new_expr);
        // Note: In the new architecture, field preservation is handled at a higher level
    }

    /// Replace action and move matching fields
    pub fn replace_action_and_move_fields(action: &mut Action, mut new_action: Action) {
        std::mem::swap(action, &mut new_action);
        // Note: In the new architecture, field preservation is handled at a higher level
    }

    /// Replace painter action and move matching fields
    pub fn replace_painter_action_and_move_fields(
        painter_action: &mut PainterAction,
        mut new_painter_action: PainterAction,
    ) {
        std::mem::swap(painter_action, &mut new_painter_action);
        // Note: In the new architecture, field preservation is handled at a higher level
    }
}
