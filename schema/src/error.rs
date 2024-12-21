use itertools::join;
use thiserror::Error;
use var_value::VarValue;

use super::*;

#[derive(Error, Debug)]
pub enum ExpressionError {
    #[error("Operation {op} for {} not supported {}", join(values, ", "), msg.clone().unwrap_or_default())]
    OperationNotSupported {
        values: Vec<VarValue>,
        op: &'static str,
        msg: Option<String>,
    },
    #[error("Value not found")]
    ValueNotFound,
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

impl ExpressionError {
    pub fn not_supported_single(op: &'static str, value: VarValue) -> Self {
        Self::OperationNotSupported {
            values: [value].into(),
            op,
            msg: None,
        }
    }
    pub fn not_supported_multiple(op: &'static str, values: Vec<VarValue>) -> Self {
        Self::OperationNotSupported {
            values,
            op,
            msg: None,
        }
    }
}
