use backtrace::Backtrace as Btrace;
use itertools::join;
use thiserror::Error;
use var_value::VarValue;

use super::*;

#[derive(Error, Debug)]
pub struct ExpressionError {
    pub source: ExpressionErrorVariants,
    pub bt: Option<Btrace>,
}

impl std::fmt::Display for ExpressionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.source)
    }
}

#[derive(Error, Debug)]
pub enum ExpressionErrorVariants {
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
            Err(e) => log::error!("{}", e.source),
        }
    }
    fn ok_log(self) -> Option<T> {
        match self {
            Ok(v) => Some(v),
            Err(e) => {
                log::error!("{}", e.source);
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
            None => Err(ExpressionErrorVariants::Custom(s.into()).into()),
        }
    }
    fn to_custom_e_fn(self, s: impl FnOnce() -> String) -> Result<T, ExpressionError> {
        match self {
            Some(v) => Ok(v),
            None => Err(ExpressionErrorVariants::Custom(s()).into()),
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
            Err(e) => Err(e.source.to_string()),
        }
    }
}

impl ExpressionError {
    pub fn not_supported_single(op: &'static str, value: VarValue) -> Self {
        ExpressionErrorVariants::OperationNotSupported {
            values: [value].into(),
            op,
            msg: None,
        }
        .into()
    }
    pub fn not_supported_multiple(op: &'static str, values: Vec<VarValue>) -> Self {
        ExpressionErrorVariants::OperationNotSupported {
            values,
            op,
            msg: None,
        }
        .into()
    }
}

impl Into<String> for ExpressionError {
    fn into(self) -> String {
        self.source.to_string()
    }
}

impl From<&str> for ExpressionError {
    fn from(value: &str) -> Self {
        ExpressionErrorVariants::Custom(value.into()).into()
    }
}
impl From<String> for ExpressionError {
    fn from(value: String) -> Self {
        ExpressionErrorVariants::Custom(value.into()).into()
    }
}
impl Into<ExpressionError> for ExpressionErrorVariants {
    fn into(self) -> ExpressionError {
        ExpressionError {
            source: self,
            bt: if true {
                None
            } else {
                Some(Btrace::new_unresolved())
            },
        }
    }
}
