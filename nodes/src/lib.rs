use serde::{Deserialize, Serialize};
use std::{fmt::Display, marker::PhantomData};

// Re-export common types that nodes will use
pub type ChildComponent<T> = Option<T>;
pub type ChildComponents<T> = Vec<T>;
pub type ParentComponent<T> = Option<T>;
pub type ParentComponents<T> = Vec<T>;

#[derive(Default, Debug, Hash, Serialize, Deserialize, PartialEq, Eq)]
pub struct ParentLinks<T> {
    pub ids: Vec<u64>,
    t: PhantomData<T>,
}

impl<T> Clone for ParentLinks<T> {
    fn clone(&self) -> Self {
        Self {
            ids: self.ids.clone(),
            t: self.t.clone(),
        }
    }
}

pub fn parent_links<T>(ids: Vec<u64>) -> ParentLinks<T> {
    ParentLinks {
        ids,
        t: PhantomData::<T>,
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct HexColor(pub String);

impl HexColor {
    pub fn new(color: &str) -> Self {
        Self(color.to_string())
    }
}

impl Default for HexColor {
    fn default() -> Self {
        Self("#ffffff".to_owned())
    }
}

impl Display for HexColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for HexColor {
    fn from(value: String) -> Self {
        Self(value)
    }
}

// This crate only provides common types
// Node definitions are generated in client/server crates

// Define the nodes! macro that will be used to import processed nodes
#[macro_export]
macro_rules! nodes {
    (client) => {
        // This will be overridden by the nodes-client crate
        pub use crate::*;
    };
    (server) => {
        // This will be overridden by the nodes-server crate
        pub use crate::*;
    };
}
