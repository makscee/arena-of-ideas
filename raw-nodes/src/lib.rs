use schema::*;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use strum::{AsRefStr, Display, EnumIter, EnumString};

#[allow(dead_code)]
mod raw_nodes;

include!(concat!(env!("OUT_DIR"), "/node_kind.rs"));
