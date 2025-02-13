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
    #[error("Value not found for {0}")]
    ValueNotFound(VarName),
    #[error("{0}")]
    Custom(String),
}

pub trait ExpressionErrorResultExt<T> {
    fn log(self);
    fn ok_log(self) -> Option<T>;
}
impl<T> ExpressionErrorResultExt<T> for Result<T, ExpressionError> {
    fn log(self) {
        match self {
            Ok(_) => {}
            Err(e) => log::error!("{e}"),
        }
    }
    fn ok_log(self) -> Option<T> {
        match self {
            Ok(v) => Some(v),
            Err(e) => {
                log::error!("{e}");
                None
            }
        }
    }
}

pub trait OptionExpressionError<T> {
    fn to_e(self, s: impl Into<String>) -> Result<T, ExpressionError>;
    fn to_e_fn(self, s: impl FnOnce() -> String) -> Result<T, ExpressionError>;
    fn to_e_s(self, s: impl Into<String>) -> Result<T, String>;
    fn to_e_s_fn(self, s: impl FnOnce() -> String) -> Result<T, String>;
    fn to_e_var(self, var: VarName) -> Result<T, ExpressionError>;
}

impl<T> OptionExpressionError<T> for Option<T> {
    fn to_e(self, s: impl Into<String>) -> Result<T, ExpressionError> {
        match self {
            Some(v) => Ok(v),
            None => Err(ExpressionError::Custom(s.into())),
        }
    }
    fn to_e_fn(self, s: impl FnOnce() -> String) -> Result<T, ExpressionError> {
        match self {
            Some(v) => Ok(v),
            None => Err(ExpressionError::Custom(s())),
        }
    }
    fn to_e_s(self, s: impl Into<String>) -> Result<T, String> {
        match self {
            Some(v) => Ok(v),
            None => Err(s.into()),
        }
    }
    fn to_e_s_fn(self, s: impl FnOnce() -> String) -> Result<T, String> {
        match self {
            Some(v) => Ok(v),
            None => Err(s()),
        }
    }
    fn to_e_var(self, var: VarName) -> Result<T, ExpressionError> {
        match self {
            Some(v) => Ok(v),
            None => Err(ExpressionError::ValueNotFound(var)),
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

impl Into<String> for ExpressionError {
    fn into(self) -> String {
        self.to_string()
    }
}

impl From<&str> for ExpressionError {
    fn from(value: &str) -> Self {
        Self::Custom(value.into())
    }
}
