use itertools::join;
use thiserror::Error;
use var_value::VarValue;

use super::*;

#[derive(Error, Debug)]
pub enum NodeError {
    #[error("Node not found: {0}")]
    NotFound(u64),

    #[error("Invalid node kind: expected {expected}, got {actual}")]
    InvalidKind {
        expected: NodeKind,
        actual: NodeKind,
    },

    #[error("Failed to load node: {0}")]
    LoadError(String),

    #[error("Failed to cast node")]
    CastError,

    #[error("Operation {op} for {} not supported {}", join(values, ", "), msg.clone().unwrap_or_default())]
    OperationNotSupported {
        values: Vec<VarValue>,
        op: &'static str,
        msg: Option<String>,
    },

    #[error("Value not found for {0}")]
    VarNotFound(VarName),

    #[error("{0}")]
    Custom(String),

    #[error("Entity#{0}_{1} not linked to id")]
    IdNotFound(u32, u32),

    #[error("Id#{0} not linked to Entity")]
    EntityNotFound(u64),

    #[error("Not found: {0}")]
    NotFoundGeneric(String),

    #[error("Context error: {0}")]
    ContextError(#[from] anyhow::Error),
}

pub type NodeResult<T> = Result<T, NodeError>;

impl NodeError {
    pub fn custom(msg: impl Into<String>) -> Self {
        NodeError::Custom(msg.into())
    }

    pub fn not_found_generic(msg: impl Into<String>) -> Self {
        NodeError::NotFoundGeneric(msg.into())
    }

    pub fn not_supported_single(op: &'static str, value: VarValue) -> Self {
        NodeError::OperationNotSupported {
            values: vec![value],
            op,
            msg: None,
        }
    }

    pub fn not_supported_multiple(op: &'static str, values: Vec<VarValue>) -> Self {
        NodeError::OperationNotSupported {
            values,
            op,
            msg: None,
        }
    }

    pub fn not_supported_with_msg(
        op: &'static str,
        values: Vec<VarValue>,
        msg: impl Into<String>,
    ) -> Self {
        NodeError::OperationNotSupported {
            values,
            op,
            msg: Some(msg.into()),
        }
    }
}

impl From<NodeError> for String {
    fn from(value: NodeError) -> Self {
        value.to_string()
    }
}

impl From<&str> for NodeError {
    fn from(value: &str) -> Self {
        NodeError::Custom(value.into())
    }
}

impl From<String> for NodeError {
    fn from(value: String) -> Self {
        NodeError::Custom(value)
    }
}

pub trait NodeErrorResultExt<T> {
    fn log(self);
    fn ok_log(self) -> Option<T>;
    fn to_str_err(self) -> Result<T, String>;
}

impl<T> NodeErrorResultExt<T> for NodeResult<T> {
    fn log(self) {
        if let Err(e) = self {
            log::error!("{}", e);
        }
    }

    fn ok_log(self) -> Option<T> {
        match self {
            Ok(v) => Some(v),
            Err(e) => {
                log::error!("{}", e);
                None
            }
        }
    }

    fn to_str_err(self) -> Result<T, String> {
        self.map_err(|e| e.to_string())
    }
}

pub trait OptionNodeExt<T> {
    fn to_custom_err(self, msg: impl Into<String>) -> NodeResult<T>;
    fn to_custom_err_fn(self, f: impl FnOnce() -> String) -> NodeResult<T>;
    fn to_not_found(self) -> NodeResult<T>;
    fn to_not_found_msg(self, msg: impl Into<String>) -> NodeResult<T>;
    fn ok_or_str(self, msg: impl Into<String>) -> Result<T, String>;
    fn ok_or_str_fn(self, f: impl FnOnce() -> String) -> Result<T, String>;
    fn to_custom_e(self, s: impl Into<String>) -> NodeResult<T>;
    fn to_custom_e_fn(self, s: impl FnOnce() -> String) -> NodeResult<T>;
    fn to_custom_e_s(self, s: impl Into<String>) -> Result<T, String>;
    fn to_custom_e_s_fn(self, s: impl FnOnce() -> String) -> Result<T, String>;
}

impl<T> OptionNodeExt<T> for Option<T> {
    fn to_custom_err(self, msg: impl Into<String>) -> NodeResult<T> {
        self.ok_or_else(|| NodeError::Custom(msg.into()))
    }

    fn to_custom_err_fn(self, f: impl FnOnce() -> String) -> NodeResult<T> {
        self.ok_or_else(|| NodeError::Custom(f()))
    }

    fn to_not_found(self) -> NodeResult<T> {
        self.ok_or_else(|| {
            NodeError::NotFoundGeneric(format!("Not found: {}", type_name_short::<T>()))
        })
    }

    fn to_not_found_msg(self, msg: impl Into<String>) -> NodeResult<T> {
        self.ok_or_else(|| NodeError::NotFoundGeneric(msg.into()))
    }

    fn ok_or_str(self, msg: impl Into<String>) -> Result<T, String> {
        self.ok_or_else(|| msg.into())
    }

    fn ok_or_str_fn(self, f: impl FnOnce() -> String) -> Result<T, String> {
        self.ok_or_else(|| f())
    }

    fn to_custom_e(self, s: impl Into<String>) -> NodeResult<T> {
        self.ok_or_else(|| NodeError::Custom(s.into()))
    }

    fn to_custom_e_fn(self, s: impl FnOnce() -> String) -> NodeResult<T> {
        self.ok_or_else(|| NodeError::Custom(s()))
    }

    fn to_custom_e_s(self, s: impl Into<String>) -> Result<T, String> {
        self.ok_or_else(|| s.into())
    }

    fn to_custom_e_s_fn(self, s: impl FnOnce() -> String) -> Result<T, String> {
        self.ok_or_else(|| s())
    }
}

pub trait VarNameNodeExt {
    fn to_var_not_found(self) -> NodeError;
}

impl VarNameNodeExt for VarName {
    fn to_var_not_found(self) -> NodeError {
        NodeError::VarNotFound(self)
    }
}

pub trait ResultNodeExt<T, E> {
    fn to_node_err(self) -> NodeResult<T>;
}

impl<T, E: std::fmt::Display> ResultNodeExt<T, E> for Result<T, E> {
    fn to_node_err(self) -> NodeResult<T> {
        self.map_err(|e| NodeError::Custom(e.to_string()))
    }
}

// Type aliases for backward compatibility during transition
pub type ExpressionError = NodeError;
pub type ExpressionErrorVariants = NodeError;

// Re-export type_name_short for internal use
pub use utils::type_name_short;
