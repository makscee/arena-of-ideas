use bevy::color::HexColorError;
use itertools::join;
use thiserror::Error;

use super::*;

#[derive(Error, Debug)]
pub enum ExpressionError {
    #[error("Operation {op} for {} not supported", join(values, ", "))]
    OperationNotSupported {
        values: Vec<VarValue>,
        op: &'static str,
    },
    #[error("Value not found")]
    ValueNotFound,
    #[error("Hex parse error: {0}")]
    HexColorError(#[from] HexColorError),
}

pub trait OptionExpressionError<T> {
    fn to_e(self) -> Result<T, ExpressionError>;
}

impl<T> OptionExpressionError<T> for Option<T> {
    fn to_e(self) -> Result<T, ExpressionError> {
        match self {
            Some(v) => Ok(v),
            None => Err(ExpressionError::ValueNotFound),
        }
    }
}
