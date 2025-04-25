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
    #[error("Entity#{0} {1} not linked to id")]
    // provide Entity on bevy 0.16 from nostd dependency
    IdNotFound(u32, u32),
    #[error("Id#{0} not liked to Entity")]
    EntityNotFound(u64),
    #[error("Not found {0}")]
    NotFound(String),
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

pub trait OptionExpressionCustomError<T> {
    fn to_custom_e(self, s: impl Into<String>) -> Result<T, ExpressionError>;
    fn to_custom_e_fn(self, s: impl FnOnce() -> String) -> Result<T, ExpressionError>;
    fn to_custom_e_s(self, s: impl Into<String>) -> Result<T, String>;
    fn to_custom_e_s_fn(self, s: impl FnOnce() -> String) -> Result<T, String>;
}

impl<T> OptionExpressionCustomError<T> for Option<T> {
    fn to_custom_e(self, s: impl Into<String>) -> Result<T, ExpressionError> {
        match self {
            Some(v) => Ok(v),
            None => Err(ExpressionError::Custom(s.into())),
        }
    }
    fn to_custom_e_fn(self, s: impl FnOnce() -> String) -> Result<T, ExpressionError> {
        match self {
            Some(v) => Ok(v),
            None => Err(ExpressionError::Custom(s())),
        }
    }
    fn to_custom_e_s(self, s: impl Into<String>) -> Result<T, String> {
        match self {
            Some(v) => Ok(v),
            None => Err(s.into()),
        }
    }
    fn to_custom_e_s_fn(self, s: impl FnOnce() -> String) -> Result<T, String> {
        match self {
            Some(v) => Ok(v),
            None => Err(s()),
        }
    }
}

pub trait ExpressionErrorString<T> {
    fn to_str_err(self) -> Result<T, String>;
}

impl<T> ExpressionErrorString<T> for Result<T, ExpressionError> {
    fn to_str_err(self) -> Result<T, String> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(e.to_string()),
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
impl From<String> for ExpressionError {
    fn from(value: String) -> Self {
        Self::Custom(value.into())
    }
}
