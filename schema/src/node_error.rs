use itertools::join;
use std::panic::Location;
use thiserror::Error;
use var_value::VarValue;

use super::*;

#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub file: &'static str,
    pub line: u32,
    pub column: u32,
}

impl SourceLocation {
    pub fn new(location: &'static Location<'static>) -> Self {
        Self {
            file: location.file(),
            line: location.line(),
            column: location.column(),
        }
    }
}

impl From<&'static Location<'static>> for SourceLocation {
    fn from(value: &'static Location<'static>) -> Self {
        Self::new(value)
    }
}

impl std::fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let file = self.file.split('/').last().unwrap_or(self.file);
        write!(f, "{}:{}:{}", file, self.line, self.column)
    }
}

#[derive(Error, Debug)]
pub enum NodeError {
    #[error("Node not found: {0} {1}")]
    NotFound(u64, SourceLocation),

    #[error("Invalid node kind: expected {expected}, got {actual} {location}")]
    InvalidKind {
        expected: NodeKind,
        actual: NodeKind,
        location: SourceLocation,
    },

    #[error("Failed to load node: {0} {1}")]
    LoadError(String, SourceLocation),

    #[error("Failed to cast node {0}")]
    CastError(SourceLocation),

    #[error("Operation {op} for {} not supported {} {location}", join(values, ", "), msg.clone().unwrap_or_default())]
    OperationNotSupported {
        values: Vec<VarValue>,
        op: &'static str,
        msg: Option<String>,
        location: SourceLocation,
    },

    #[error("Value not found for {0} {1}")]
    VarNotFound(VarName, SourceLocation),

    #[error("{0} {1}")]
    Custom(String, SourceLocation),

    #[error("Entity#{0}_{1} not linked to id {2}")]
    IdNotFound(u32, u32, SourceLocation),

    #[error("Id#{0} not linked to Entity {1}")]
    EntityNotFound(u64, SourceLocation),

    #[error("Not found: {0} {1}")]
    NotFoundGeneric(String, SourceLocation),

    #[error("Context error: {0} {1}")]
    ContextError(anyhow::Error, SourceLocation),
}

pub type NodeResult<T> = Result<T, NodeError>;

impl NodeError {
    #[track_caller]
    pub fn not_found(id: u64) -> Self {
        NodeError::NotFound(id, Location::caller().into())
    }

    #[track_caller]
    pub fn invalid_kind(expected: NodeKind, actual: NodeKind) -> Self {
        NodeError::InvalidKind {
            expected,
            actual,
            location: Location::caller().into(),
        }
    }

    #[track_caller]
    pub fn load_error(msg: impl Into<String>) -> Self {
        NodeError::LoadError(msg.into(), Location::caller().into())
    }

    #[track_caller]
    pub fn cast_error() -> Self {
        NodeError::CastError(Location::caller().into())
    }

    #[track_caller]
    pub fn var_not_found(var: VarName) -> Self {
        NodeError::VarNotFound(var, Location::caller().into())
    }

    #[track_caller]
    pub fn id_not_found(entity_high: u32, entity_low: u32) -> Self {
        NodeError::IdNotFound(entity_high, entity_low, Location::caller().into())
    }

    #[track_caller]
    pub fn entity_not_found(id: u64) -> Self {
        NodeError::EntityNotFound(id, Location::caller().into())
    }

    #[track_caller]
    pub fn context_error(error: anyhow::Error) -> Self {
        NodeError::ContextError(error, Location::caller().into())
    }

    #[track_caller]
    pub fn custom(msg: impl Into<String>) -> Self {
        NodeError::Custom(msg.into(), Location::caller().into())
    }

    #[track_caller]
    pub fn not_found_generic(msg: impl Into<String>) -> Self {
        NodeError::NotFoundGeneric(msg.into(), Location::caller().into())
    }

    #[track_caller]
    pub fn not_supported_single(op: &'static str, value: VarValue) -> Self {
        NodeError::OperationNotSupported {
            values: vec![value],
            op,
            msg: None,
            location: Location::caller().into(),
        }
    }

    #[track_caller]
    pub fn not_supported_multiple(op: &'static str, values: Vec<VarValue>) -> Self {
        NodeError::OperationNotSupported {
            values,
            op,
            msg: None,
            location: Location::caller().into(),
        }
    }

    #[track_caller]
    pub fn not_supported_with_msg(
        op: &'static str,
        values: Vec<VarValue>,
        msg: impl Into<String>,
    ) -> Self {
        NodeError::OperationNotSupported {
            values,
            op,
            msg: Some(msg.into()),
            location: Location::caller().into(),
        }
    }
}

impl From<NodeError> for String {
    fn from(value: NodeError) -> Self {
        value.to_string()
    }
}

impl From<anyhow::Error> for NodeError {
    #[track_caller]
    fn from(error: anyhow::Error) -> Self {
        NodeError::context_error(error)
    }
}

impl From<&str> for NodeError {
    #[track_caller]
    fn from(value: &str) -> Self {
        NodeError::custom(value)
    }
}

impl From<String> for NodeError {
    #[track_caller]
    fn from(value: String) -> Self {
        NodeError::custom(value)
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
            NodeError::log(&e);
        }
    }

    fn ok_log(self) -> Option<T> {
        match self {
            Ok(v) => Some(v),
            Err(e) => {
                NodeError::log(&e);
                None
            }
        }
    }

    fn to_str_err(self) -> Result<T, String> {
        self.map_err(|e| e.to_string())
    }
}

impl NodeError {
    pub fn log(&self) {
        log::error!("{}", self);
    }
}

pub trait OptionNodeExt<T> {
    #[track_caller]
    fn to_custom_err(self, msg: impl Into<String>) -> NodeResult<T>;
    #[track_caller]
    fn to_custom_err_fn(self, f: impl FnOnce() -> String) -> NodeResult<T>;
    #[track_caller]
    fn to_not_found(self) -> NodeResult<T>;
    #[track_caller]
    fn to_not_found_msg(self, msg: impl Into<String>) -> NodeResult<T>;
    fn ok_or_str(self, msg: impl Into<String>) -> Result<T, String>;
    fn ok_or_str_fn(self, f: impl FnOnce() -> String) -> Result<T, String>;
    #[track_caller]
    fn to_custom_e(self, s: impl Into<String>) -> NodeResult<T>;
    #[track_caller]
    fn to_custom_e_fn(self, s: impl FnOnce() -> String) -> NodeResult<T>;
    fn to_custom_e_s(self, s: impl Into<String>) -> Result<T, String>;
    fn to_custom_e_s_fn(self, s: impl FnOnce() -> String) -> Result<T, String>;
}

impl<T> OptionNodeExt<T> for Option<T> {
    #[track_caller]
    fn to_custom_err(self, msg: impl Into<String>) -> NodeResult<T> {
        self.ok_or_else(|| NodeError::custom(msg))
    }

    #[track_caller]
    fn to_custom_err_fn(self, f: impl FnOnce() -> String) -> NodeResult<T> {
        self.ok_or_else(|| NodeError::custom(f()))
    }

    #[track_caller]
    fn to_not_found(self) -> NodeResult<T> {
        self.ok_or_else(|| {
            NodeError::not_found_generic(format!("Not found: {}", type_name_short::<T>()))
        })
    }

    #[track_caller]
    fn to_not_found_msg(self, msg: impl Into<String>) -> NodeResult<T> {
        self.ok_or_else(|| NodeError::not_found_generic(msg))
    }

    fn ok_or_str(self, msg: impl Into<String>) -> Result<T, String> {
        self.ok_or_else(|| msg.into())
    }

    fn ok_or_str_fn(self, f: impl FnOnce() -> String) -> Result<T, String> {
        self.ok_or_else(|| f())
    }

    #[track_caller]
    fn to_custom_e(self, s: impl Into<String>) -> NodeResult<T> {
        self.ok_or_else(|| NodeError::custom(s))
    }

    #[track_caller]
    fn to_custom_e_fn(self, s: impl FnOnce() -> String) -> NodeResult<T> {
        self.ok_or_else(|| NodeError::custom(s()))
    }

    fn to_custom_e_s(self, s: impl Into<String>) -> Result<T, String> {
        self.ok_or_else(|| s.into())
    }

    fn to_custom_e_s_fn(self, s: impl FnOnce() -> String) -> Result<T, String> {
        self.ok_or_else(|| s())
    }
}

pub trait VarNameNodeExt {
    #[track_caller]
    fn to_var_not_found(self) -> NodeError;
}

impl VarNameNodeExt for VarName {
    #[track_caller]
    fn to_var_not_found(self) -> NodeError {
        NodeError::var_not_found(self)
    }
}

pub trait ResultNodeExt<T, E> {
    #[track_caller]
    fn to_node_err(self) -> NodeResult<T>;
}

impl<T, E: std::fmt::Display> ResultNodeExt<T, E> for Result<T, E> {
    #[track_caller]
    fn to_node_err(self) -> NodeResult<T> {
        self.map_err(|e| NodeError::custom(e.to_string()))
    }
}
