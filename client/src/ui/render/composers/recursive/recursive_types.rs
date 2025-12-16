use super::*;

pub struct RecursiveField<'a> {
    pub name: String,
    pub value: RecursiveValue<'a>,
}

#[derive(Clone, Debug)]
pub enum RecursiveValue<'a> {
    Expr(&'a Expression),
    Action(&'a Action),
    Var(&'a VarName),
    VarValue(&'a VarValue),
    HexColor(&'a HexColor),
    String(&'a String),
    I32(&'a i32),
    F32(&'a f32),
    Bool(&'a bool),
    Vec2(&'a Vec2),
    Behavior(&'a Behavior),
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
    Var(&'a mut VarName),
    VarValue(&'a mut VarValue),
    HexColor(&'a mut HexColor),
    String(&'a mut String),
    I32(&'a mut i32),
    F32(&'a mut f32),
    Bool(&'a mut bool),
    Vec2(&'a mut Vec2),
    Behavior(&'a mut Behavior),
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
macro_rules! recursive_value_match {
    ($value: expr, $v: ident, $code: expr) => {
        match $value {
            RecursiveValue::Expr($v) => $code,
            RecursiveValue::Action($v) => $code,
            RecursiveValue::Var($v) => $code,
            RecursiveValue::VarValue($v) => $code,
            RecursiveValue::HexColor($v) => $code,
            RecursiveValue::String($v) => $code,
            RecursiveValue::I32($v) => $code,
            RecursiveValue::F32($v) => $code,
            RecursiveValue::Bool($v) => $code,
            RecursiveValue::Vec2($v) => $code,
            RecursiveValue::Behavior($v) => $code,
            RecursiveValue::Material($v) => $code,
        }
    };
}

#[macro_export]
macro_rules! call_on_recursive_value {
    ($value:expr, $func:ident $(, $arg:expr)*) => {
        match $value {
            RecursiveValue::Expr(v) => v.$func($($arg),*),
            RecursiveValue::Action(v) => v.$func($($arg),*),
            RecursiveValue::Var(v) => v.$func($($arg),*),
            RecursiveValue::VarValue(v) => v.$func($($arg),*),
            RecursiveValue::HexColor(v) => v.$func($($arg),*),
            RecursiveValue::String(v) => v.$func($($arg),*),
            RecursiveValue::I32(v) => v.$func($($arg),*),
            RecursiveValue::F32(v) => v.$func($($arg),*),
            RecursiveValue::Bool(v) => v.$func($($arg),*),
            RecursiveValue::Vec2(v) => v.$func($($arg),*),
            RecursiveValue::Behavior(v) => v.$func($($arg),*),
            RecursiveValue::Material(v) => v.$func($($arg),*),
        }
    };
}

#[macro_export]
macro_rules! call_pass_recursive_value {
    ($value:expr, $func:ident $(, $arg:expr)*) => {
        match $value {
            RecursiveValue::Expr(v) => $func(v, $($arg),*),
            RecursiveValue::Action(v) => $func(v, $($arg),*),
            RecursiveValue::Var(v) => $func(v, $($arg),*),
            RecursiveValue::VarValue(v) => $func(v, $($arg),*),
            RecursiveValue::HexColor(v) => $func(v, $($arg),*),
            RecursiveValue::String(v) => $func(v, $($arg),*),
            RecursiveValue::I32(v) => $func(v, $($arg),*),
            RecursiveValue::F32(v) => $func(v, $($arg),*),
            RecursiveValue::Bool(v) => $func(v, $($arg),*),
            RecursiveValue::Vec2(v) => $func(v, $($arg),*),
            RecursiveValue::Behavior(v) => $func(v, $($arg),*),
            RecursiveValue::Material(v) => $func(v, $($arg),*),
        }
    };
}

#[macro_export]
macro_rules! call_on_recursive_value_mut {
    ($value:expr, $func:ident $(, $arg:expr)*) => {
        match $value {
            RecursiveValueMut::Expr(v) => v.$func($($arg),*),
            RecursiveValueMut::Action(v) => v.$func($($arg),*),
            RecursiveValueMut::Var(v) => v.$func($($arg),*),
            RecursiveValueMut::VarValue(v) => v.$func($($arg),*),
            RecursiveValueMut::HexColor(v) => v.$func($($arg),*),
            RecursiveValueMut::String(v) => v.$func($($arg),*),
            RecursiveValueMut::I32(v) => v.$func($($arg),*),
            RecursiveValueMut::F32(v) => v.$func($($arg),*),
            RecursiveValueMut::Bool(v) => v.$func($($arg),*),
            RecursiveValueMut::Vec2(v) => v.$func($($arg),*),
            RecursiveValueMut::Behavior(v) => v.$func($($arg),*),
            RecursiveValueMut::Material(v) => v.$func($($arg),*),
        }
    };
}

#[macro_export]
macro_rules! call_pass_recursive_value_mut {
    ($value:expr, $func:ident $(, $arg:expr)*) => {
        match $value {
            RecursiveValueMut::Expr(v) => $func(v, $($arg),*),
            RecursiveValueMut::Action(v) => $func(v, $($arg),*),
            RecursiveValueMut::Var(v) => $func(v, $($arg),*),
            RecursiveValueMut::VarValue(v) => $func(v, $($arg),*),
            RecursiveValueMut::HexColor(v) => $func(v, $($arg),*),
            RecursiveValueMut::String(v) => $func(v, $($arg),*),
            RecursiveValueMut::I32(v) => $func(v, $($arg),*),
            RecursiveValueMut::F32(v) => $func(v, $($arg),*),
            RecursiveValueMut::Bool(v) => $func(v, $($arg),*),
            RecursiveValueMut::Vec2(v) => $func(v, $($arg),*),
            RecursiveValueMut::Behavior(v) => $func(v, $($arg),*),
            RecursiveValueMut::Material(v) => $func(v, $($arg),*),
        }
    };
}

impl<'a> RecursiveValueMut<'a> {
    /// Attempts to move one RecursiveFieldMut into another by matching field types.
    /// Returns true if a move happened.
    fn try_move_field_by_type<'b>(
        source_field: &mut RecursiveFieldMut<'b>,
        target_field: &mut RecursiveFieldMut<'b>,
    ) -> bool {
        use std::mem;

        match (&mut source_field.value, &mut target_field.value) {
            (RecursiveValueMut::Expr(source), RecursiveValueMut::Expr(target)) => {
                mem::swap(*source, *target);
                true
            }
            (RecursiveValueMut::Action(source), RecursiveValueMut::Action(target)) => {
                mem::swap(*source, *target);
                true
            }
            (RecursiveValueMut::Var(source), RecursiveValueMut::Var(target)) => {
                mem::swap(*source, *target);
                true
            }
            (RecursiveValueMut::VarValue(source), RecursiveValueMut::VarValue(target)) => {
                mem::swap(*source, *target);
                true
            }
            (RecursiveValueMut::HexColor(source), RecursiveValueMut::HexColor(target)) => {
                mem::swap(*source, *target);
                true
            }
            (RecursiveValueMut::String(source), RecursiveValueMut::String(target)) => {
                mem::swap(*source, *target);
                true
            }
            (RecursiveValueMut::I32(source), RecursiveValueMut::I32(target)) => {
                mem::swap(*source, *target);
                true
            }
            (RecursiveValueMut::F32(source), RecursiveValueMut::F32(target)) => {
                mem::swap(*source, *target);
                true
            }
            (RecursiveValueMut::Bool(source), RecursiveValueMut::Bool(target)) => {
                mem::swap(*source, *target);
                true
            }
            (RecursiveValueMut::Vec2(source), RecursiveValueMut::Vec2(target)) => {
                mem::swap(*source, *target);
                true
            }
            (RecursiveValueMut::Behavior(source), RecursiveValueMut::Behavior(target)) => {
                mem::swap(*source, *target);
                true
            }
            (RecursiveValueMut::Material(source), RecursiveValueMut::Material(target)) => {
                mem::swap(*source, *target);
                true
            }
            _ => false,
        }
    }
}

/// Extension trait to add move_from method to Vec<RecursiveFieldMut>
pub trait FieldsMover<'a> {
    fn move_from(&mut self, source: &'a mut impl FRecursive);
}

impl<'a> FieldsMover<'a> for Vec<RecursiveFieldMut<'a>> {
    fn move_from(&mut self, source: &'a mut impl FRecursive) {
        for source_field in source.get_inner_fields_mut().iter_mut() {
            for i in 0..self.len() {
                if RecursiveValueMut::try_move_field_by_type(source_field, &mut self[i]) {
                    self.remove(i);
                    break;
                }
            }
        }
    }
}
