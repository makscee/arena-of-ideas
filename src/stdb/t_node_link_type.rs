// THIS FILE IS AUTOMATICALLY GENERATED BY SPACETIMEDB. EDITS TO THIS FILE
// WILL NOT BE SAVED. MODIFY TABLES IN YOUR MODULE SOURCE CODE INSTEAD.

#![allow(unused, clippy::all)]
use spacetimedb_sdk::__codegen::{self as __sdk, __lib, __sats, __ws};

#[derive(__lib::ser::Serialize, __lib::de::Deserialize, Clone, PartialEq, Debug)]
#[sats(crate = __lib)]
pub struct TNodeLink {
    pub child: u64,
    pub parent: u64,
    pub child_kind: String,
    pub parent_kind: String,
    pub score: i32,
}

impl __sdk::InModule for TNodeLink {
    type Module = super::RemoteModule;
}
