use schema::{ExpressionErrorVariants, type_name_short};

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
            None => Err(ExpressionErrorVariants::ValueNotFound(f).into()),
        }
    }
}
impl ToEParam<Entity, u64> for Option<u64> {
    fn to_e(self, f: Entity) -> Result<u64, ExpressionError> {
        match self {
            Some(v) => Ok(v),
            None => Err(ExpressionErrorVariants::IdNotFound(f.index(), f.generation()).into()),
        }
    }
}
impl ToEParam<u64, Entity> for Option<Entity> {
    fn to_e(self, f: u64) -> Result<Entity, ExpressionError> {
        match self {
            Some(v) => Ok(v),
            None => Err(ExpressionErrorVariants::EntityNotFound(f).into()),
        }
    }
}

impl<'a> ToE<&'a World> for Option<&'a World> {
    fn to_e(self) -> Result<&'a World, ExpressionError> {
        match self {
            Some(v) => Ok(v),
            None => Err(ExpressionErrorVariants::Custom("World not found".into()).into()),
        }
    }
}
impl<'a> ToE<&'a mut World> for Option<&'a mut World> {
    fn to_e(self) -> Result<&'a mut World, ExpressionError> {
        match self {
            Some(v) => Ok(v),
            None => Err(ExpressionErrorVariants::Custom("World not found".into()).into()),
        }
    }
}

impl<T> ToENotFound<T> for Option<T> {
    fn to_e_not_found(self) -> Result<T, ExpressionError> {
        match self {
            Some(v) => Ok(v),
            None => Err(ExpressionErrorVariants::NotFound(format!(
                "Not found: {}",
                type_name_short::<T>()
            ))
            .into()),
        }
    }
}
