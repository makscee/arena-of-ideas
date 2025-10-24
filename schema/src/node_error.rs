use itertools::Itertools;
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
                let file = if let Some(pos) = loc.file().find("arena-of-ideas/") {
                    &loc.file()[pos + "arena-of-ideas/".len()..]
                } else {
                    loc.file().split('/').last().unwrap_or(loc.file())
                };
                format!("{}:{}:{}", file, loc.line(), loc.column())
            })
            .collect();
        write!(f, "{}", location_strs.join(" <- "))
    }
}

#[derive(Error, Debug)]
pub enum NodeError {
    #[error("[red Node not found:] {0} {1}")]
    NotFound(u64, SourceTrace),

    #[error("[red Linked node not found:] node {node_id} has no {kind} link {location}")]
    LinkedNodeNotFound {
        node_id: u64,
        kind: NodeKind,
        location: SourceTrace,
    },

    #[error("[red Invalid node kind:] expected {expected}, got {actual} {location}")]
    InvalidKind {
        expected: NodeKind,
        actual: NodeKind,
        location: SourceTrace,
    },

    #[error("[red Failed to load node:] {0} {1}")]
    LoadError(String, SourceTrace),

    #[error("[red Failed to cast node] {0}")]
    CastError(SourceTrace),

    #[error("[red Operation {op} for {} not supported {}] {location}", values.iter().map(|v| format!("{v:?}")).join(", "), msg.clone().unwrap_or_default())]
    OperationNotSupported {
        values: Vec<VarValue>,
        op: &'static str,
        msg: Option<String>,
        location: SourceTrace,
    },

    #[error("[red Value not found for {0}] {1}")]
    VarNotFound(VarName, SourceTrace),

    #[error("[red {0}] {1}")]
    Custom(String, SourceTrace),

    #[error("[red Entity#{0}_{1} not linked to id] {2}")]
    IdNotFound(u32, u32, SourceTrace),

    #[error("[red Id#{0} not linked to Entity] {1}")]
    EntityNotFound(u64, SourceTrace),

    #[error("[red Not found: {0}] {1}")]
    NotFoundGeneric(String, SourceTrace),

    #[error("[red Context error: {0}] {1}")]
    ContextError(anyhow::Error, SourceTrace),
}

pub type NodeResult<T> = Result<T, NodeError>;

impl NodeError {
    #[track_caller]
    pub fn not_found(id: u64) -> Self {
        NodeError::NotFound(id, Location::caller().into())
    }

    #[track_caller]
    pub fn linked_node_not_found(node_id: u64, kind: NodeKind) -> Self {
        NodeError::LinkedNodeNotFound {
            node_id,
            kind,
            location: Location::caller().into(),
        }
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
    #[track_caller]
    fn track(self) -> Self;
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
                NodeError::LinkedNodeNotFound { location, .. } => {
                    *location = location.clone().add_location(current_location);
                }
                NodeError::InvalidKind { location, .. } => {
                    *location = location.clone().add_location(current_location);
                }
                NodeError::LoadError(_, trace) => {
                    *trace = trace.clone().add_location(current_location);
                }
                NodeError::CastError(trace) => {
                    *trace = trace.clone().add_location(current_location);
                }
                NodeError::OperationNotSupported { location, .. } => {
                    *location = location.clone().add_location(current_location);
                }
                NodeError::VarNotFound(_, trace) => {
                    *trace = trace.clone().add_location(current_location);
                }
                NodeError::Custom(_, trace) => {
                    *trace = trace.clone().add_location(current_location);
                }
                NodeError::IdNotFound(_, _, trace) => {
                    *trace = trace.clone().add_location(current_location);
                }
                NodeError::EntityNotFound(_, trace) => {
                    *trace = trace.clone().add_location(current_location);
                }
                NodeError::NotFoundGeneric(_, trace) => {
                    *trace = trace.clone().add_location(current_location);
                }
                NodeError::ContextError(_, trace) => {
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
    fn to_custom_err(self, msg: impl Into<String>) -> NodeResult<T>;
    #[track_caller]
    fn to_custom_err_fn(self, f: impl FnOnce() -> String) -> NodeResult<T>;
    #[track_caller]
    fn to_not_found(self) -> NodeResult<T>;
    #[track_caller]
    fn to_not_found_msg(self, msg: impl Into<String>) -> NodeResult<T>;
    fn to_not_found_id(self, id: u64) -> NodeResult<T>;
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
        self.ok_or_else(|| NodeError::not_found_generic(type_name_short::<T>()))
    }

    #[track_caller]
    fn to_not_found_msg(self, msg: impl Into<String>) -> NodeResult<T> {
        self.ok_or_else(|| NodeError::not_found_generic(msg))
    }

    fn to_not_found_id(self, id: u64) -> NodeResult<T> {
        self.ok_or_else(|| NodeError::not_found(id))
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
