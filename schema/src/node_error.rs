use itertools::Itertools;
use rhai::{Dynamic, Variant};
use std::panic::Location;
use thiserror::Error;
use var_value::VarValue;

use super::*;

#[derive(Debug, Clone)]
pub struct SourceTrace {
    locations: Vec<&'static Location<'static>>,
}

impl SourceTrace {
    pub fn new(location: &'static Location<'static>) -> Self {
        Self {
            locations: vec![location],
        }
    }

    pub fn with_locations(locations: Vec<&'static Location<'static>>) -> Self {
        Self { locations }
    }

    pub fn add_location(mut self, location: &'static Location<'static>) -> Self {
        self.locations.push(location);
        self
    }
}

impl From<&'static Location<'static>> for SourceTrace {
    fn from(value: &'static Location<'static>) -> Self {
        Self::new(value)
    }
}

impl std::fmt::Display for SourceTrace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let location_strs: Vec<String> = self
            .locations
            .iter()
            .map(|loc| {
                let file_path = loc.file();
                let file = if file_path.len() > 15 {
                    let truncated = &file_path[file_path.len() - 15..];
                    if let Some(pos) = truncated.find("/") {
                        &truncated[pos + 1..]
                    } else {
                        truncated
                    }
                } else if let Some(pos) = file_path.rfind("/") {
                    &file_path[pos + 1..]
                } else {
                    file_path
                };
                format!("{}:{}:{}", file, loc.line(), loc.column())
            })
            .collect();
        write!(f, "{}", location_strs.join(" <- "))
    }
}

#[derive(Error, Debug, Clone)]
pub enum NodeError {
    #[error("Node not found: {0} {1}")]
    NotFound(u64, SourceTrace),

    #[error("Entity not found: {0} {1}")]
    EntityNotFound(u64, SourceTrace),

    #[error("Variable not found: {0} {1}")]
    VarNotFound(VarName, SourceTrace),

    #[error("Invalid state: {context} {location}")]
    InvalidState {
        context: String,
        location: SourceTrace,
    },

    #[error("Not in context: {0} {1}")]
    NotInContext(String, SourceTrace),

    #[error("Operation {op} not supported {location}")]
    OperationNotSupported {
        values: Vec<VarValue>,
        op: &'static str,
        msg: Option<String>,
        location: SourceTrace,
    },

    #[error("{0} {1}")]
    Custom(String, SourceTrace),
}

pub type NodeResult<T> = Result<T, NodeError>;

pub trait ToDynamicResult {
    fn dynamic_result(&self) -> Dynamic;
}

impl<T: Clone + Variant> ToDynamicResult for NodeResult<T> {
    fn dynamic_result(&self) -> Dynamic {
        match self {
            Ok(value) => value.to_dynamic(),
            Err(_) => Dynamic::UNIT,
        }
    }
}

pub trait ToDynamic {
    fn to_dynamic(&self) -> Dynamic;
}

impl<T: Clone + Variant> ToDynamic for T {
    fn to_dynamic(&self) -> Dynamic {
        Dynamic::from(self.clone())
    }
}

impl NodeError {
    #[track_caller]
    pub fn not_found(id: u64) -> Self {
        NodeError::NotFound(id, Location::caller().into())
    }

    #[track_caller]
    pub fn var_not_found(var: VarName) -> Self {
        NodeError::VarNotFound(var, Location::caller().into())
    }

    #[track_caller]
    pub fn entity_not_found(id: u64) -> Self {
        NodeError::EntityNotFound(id, Location::caller().into())
    }

    #[track_caller]
    pub fn custom(msg: impl Into<String>) -> Self {
        NodeError::Custom(msg.into(), Location::caller().into())
    }

    #[track_caller]
    pub fn not_in_context(msg: impl Into<String>) -> Self {
        NodeError::NotInContext(msg.into(), Location::caller().into())
    }

    #[track_caller]
    pub fn invalid_state(context: impl Into<String>) -> Self {
        NodeError::InvalidState {
            context: context.into(),
            location: Location::caller().into(),
        }
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
    #[track_caller]
    fn track(self) -> Self;
    #[track_caller]
    fn with_context(self, msg: impl Into<String>) -> Self;
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

    #[track_caller]
    fn track(self) -> Self {
        let current_location = Location::caller();
        self.map_err(|mut e| {
            match &mut e {
                NodeError::NotFound(_, trace) => {
                    *trace = trace.clone().add_location(current_location);
                }
                NodeError::EntityNotFound(_, trace) => {
                    *trace = trace.clone().add_location(current_location);
                }
                NodeError::VarNotFound(_, trace) => {
                    *trace = trace.clone().add_location(current_location);
                }
                NodeError::InvalidState { location, .. } => {
                    *location = location.clone().add_location(current_location);
                }
                NodeError::NotInContext(_, trace) => {
                    *trace = trace.clone().add_location(current_location);
                }
                NodeError::OperationNotSupported { location, .. } => {
                    *location = location.clone().add_location(current_location);
                }
                NodeError::Custom(_, trace) => {
                    *trace = trace.clone().add_location(current_location);
                }
            }
            e
        })
    }

    #[track_caller]
    fn with_context(self, _msg: impl Into<String>) -> Self {
        let current_location = Location::caller();
        self.map_err(|mut e| {
            match &mut e {
                NodeError::NotFound(_, trace) => {
                    *trace = trace.clone().add_location(current_location);
                }
                NodeError::EntityNotFound(_, trace) => {
                    *trace = trace.clone().add_location(current_location);
                }
                NodeError::VarNotFound(_, trace) => {
                    *trace = trace.clone().add_location(current_location);
                }
                NodeError::InvalidState { location, .. } => {
                    *location = location.clone().add_location(current_location);
                }
                NodeError::NotInContext(_, trace) => {
                    *trace = trace.clone().add_location(current_location);
                }
                NodeError::OperationNotSupported { location, .. } => {
                    *location = location.clone().add_location(current_location);
                }
                NodeError::Custom(_, trace) => {
                    *trace = trace.clone().add_location(current_location);
                }
            }
            e
        })
    }
}

impl NodeError {
    pub fn log(&self) {
        log::error!("{}", self);
    }
}

pub trait OptionNodeExt<T> {
    #[track_caller]
    fn to_not_found(self) -> NodeResult<T>;
    #[track_caller]
    fn to_var_not_found(self, var: VarName) -> NodeResult<T>;
    #[track_caller]
    fn not_in_context(self, context: impl Into<String>) -> NodeResult<T>;
}

impl<T> OptionNodeExt<T> for Option<T> {
    #[track_caller]
    fn to_not_found(self) -> NodeResult<T> {
        self.ok_or_else(|| NodeError::custom(format!("{} not found", type_name_short::<T>())))
    }

    #[track_caller]
    fn to_var_not_found(self, var: VarName) -> NodeResult<T> {
        self.ok_or_else(|| NodeError::var_not_found(var))
    }

    #[track_caller]
    fn not_in_context(self, context: impl Into<String>) -> NodeResult<T> {
        self.ok_or_else(|| NodeError::not_in_context(context))
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

#[macro_export]
macro_rules! bail {
    ($msg:expr) => {
        return Err($crate::NodeError::custom($msg))
    };
}

#[macro_export]
macro_rules! bail_not_found {
    ($id:expr) => {
        return Err($crate::NodeError::not_found($id))
    };
}

#[macro_export]
macro_rules! bail_var {
    ($var:expr) => {
        return Err($crate::NodeError::var_not_found($var))
    };
}

#[macro_export]
macro_rules! bail_not_in_context {
    ($msg:expr) => {
        return Err($crate::NodeError::not_in_context($msg))
    };
}
