use std::any::type_name;

use super::*;

pub trait ToEParam<F, T> {
    fn to_e(self, f: F) -> Result<T, ExpressionError>;
}
pub trait ToE<T> {
    fn to_e(self) -> Result<T, ExpressionError>;
}
pub trait ToENotFound<T> {
    fn to_e_not_found(self) -> Result<T, ExpressionError>;
}

impl ToEParam<VarName, VarValue> for Option<VarValue> {
    fn to_e(self, f: VarName) -> Result<VarValue, ExpressionError> {
        match self {
            Some(v) => Ok(v),
            None => Err(ExpressionError::ValueNotFound(f)),
        }
    }
}
impl ToEParam<Entity, u64> for Option<u64> {
    fn to_e(self, f: Entity) -> Result<u64, ExpressionError> {
        match self {
            Some(v) => Ok(v),
            None => Err(ExpressionError::IdNotFound(f.index(), f.generation())),
        }
    }
}
impl ToEParam<u64, Entity> for Option<Entity> {
    fn to_e(self, f: u64) -> Result<Entity, ExpressionError> {
        match self {
            Some(v) => Ok(v),
            None => Err(ExpressionError::EntityNotFound(f)),
        }
    }
}

impl<'a> ToE<&'a World> for Option<&'a World> {
    fn to_e(self) -> Result<&'a World, ExpressionError> {
        match self {
            Some(v) => Ok(v),
            None => Err(ExpressionError::Custom("World not found".into())),
        }
    }
}
impl<'a> ToE<&'a mut World> for Option<&'a mut World> {
    fn to_e(self) -> Result<&'a mut World, ExpressionError> {
        match self {
            Some(v) => Ok(v),
            None => Err(ExpressionError::Custom("World not found".into())),
        }
    }
}

impl<T> ToENotFound<T> for Option<T> {
    fn to_e_not_found(self) -> Result<T, ExpressionError> {
        match self {
            Some(v) => Ok(v),
            None => Err(ExpressionError::NotFound(format!(
                "Not found: {}",
                type_name::<T>()
            ))),
        }
    }
}
