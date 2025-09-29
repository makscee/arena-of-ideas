pub mod links;
pub mod raw_nodes;

pub use links::*;

use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, Display, EnumIter, EnumString};

// Re-export types from schema crate
pub use schema::{
    Action, HexColor, MagicType, Material, Reaction, ShopOffer, Trigger, UnitActionRange,
};

// Re-export generated code
include!(concat!(env!("OUT_DIR"), "/node_kind.rs"));

// Import the node macro from proc-macros
pub use proc_macros::node;
