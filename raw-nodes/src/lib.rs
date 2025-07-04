use schema::*;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use strum::{AsRefStr, Display, EnumIter, EnumString, IntoEnumIterator};

#[allow(dead_code)]
mod raw_nodes;

include!(concat!(env!("OUT_DIR"), "/node_kind.rs"));

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    AsRefStr,
    Display,
    EnumIter,
    EnumString,
    Serialize,
    Deserialize,
)]
pub enum NodeKindCategory {
    Unit,
    House,
    Ability,
    Status,
    Other,
}

impl NodeKind {
    pub fn category(self) -> NodeKindCategory {
        match self {
            NodeKind::NUnit
            | NodeKind::NUnitStats
            | NodeKind::NUnitDescription
            | NodeKind::NUnitRepresentation
            | NodeKind::NUnitBehavior => NodeKindCategory::Unit,
            NodeKind::NHouse | NodeKind::NHouseColor => NodeKindCategory::House,
            NodeKind::NActionAbility | NodeKind::NActionDescription | NodeKind::NActionEffect => {
                NodeKindCategory::Ability
            }
            NodeKind::NStatusAbility
            | NodeKind::NStatusDescription
            | NodeKind::NStatusBehavior
            | NodeKind::NStatusRepresentation => NodeKindCategory::Status,
            _ => NodeKindCategory::Other,
        }
    }
}

impl NodeKindCategory {
    pub fn kinds(self) -> Vec<NodeKind> {
        NodeKind::iter().filter(|k| k.category() == self).collect()
    }
}
