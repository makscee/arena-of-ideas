mod action;
mod context;
mod event;
mod expression;
mod fusion;
mod links;
mod r#match;
mod node_assets;
mod node_error;
mod packed_nodes;
mod painter_action;
mod reaction;
mod tier;
mod trigger;
mod var_name;
mod var_value;

#[allow(dead_code)]
mod raw_nodes;

// Re-export node macro and types from raw_nodes
pub use proc_macros::Node;

use std::{fmt::Display, str::FromStr};

pub use action::*;
pub use context::*;
use ecolor::Color32;
pub use event::*;
pub use expression::*;
pub use fusion::*;
pub use links::*;
pub use r#match::*;
pub use node_assets::*;
#[allow(unused_imports)]
pub use node_error::*;

pub use packed_nodes::*;
pub use painter_action::*;
pub use reaction::*;
use ron::ser::to_string;
pub use tier::*;
pub use trigger::*;
pub use var_name::*;
pub use var_value::*;

pub use glam::Vec2;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use strum_macros::{AsRefStr, Display, EnumIter, EnumString};
pub use thiserror::Error;
pub use utils::*;

pub const ID_CORE: u64 = 1;
pub const ID_PLAYERS: u64 = 2;
pub const ID_ARENA: u64 = 3;

pub const NODE_CONTAINERS: [u64; 3] = [ID_CORE, ID_PLAYERS, ID_ARENA];

// Include generated NodeKind enum
include!(concat!(env!("OUT_DIR"), "/node_kind.rs"));

pub trait ToNodeKind {
    fn to_kind(&self) -> NodeKind;
}

impl ToNodeKind for String {
    fn to_kind(&self) -> NodeKind {
        NodeKind::from_str(self.as_str()).unwrap()
    }
}

impl NodeKind {
    pub fn to_named(self) -> NodeResult<NamedNodeKind> {
        if self.is_named() {
            Ok(self.try_into().unwrap())
        } else {
            Err(format!("NodeKind is not named").into())
        }
    }
    pub fn with_other_components(self) -> HashSet<NodeKind> {
        let mut set = self.other_components();
        set.insert(self);
        set
    }
}

pub trait StringData: Sized {
    fn inject_data(&mut self, data: &str) -> NodeResult<()>;
    fn get_data(&self) -> String;
}
impl<T> StringData for T
where
    T: Serialize + DeserializeOwned,
{
    fn inject_data(&mut self, data: &str) -> NodeResult<()> {
        match ron::from_str(data) {
            Ok(v) => {
                *self = v;
                Ok(())
            }
            Err(e) => Err(format!("Deserialize error: {e}").into()),
        }
    }
    fn get_data(&self) -> String {
        to_string(self).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct HexColor(pub String);

impl HexColor {
    pub fn c32(&self) -> Color32 {
        self.into()
    }
    pub fn try_c32(&self) -> Result<Color32, ecolor::ParseHexColorError> {
        Color32::from_hex(&self.0)
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

impl From<Color32> for HexColor {
    fn from(value: Color32) -> Self {
        Self(ecolor::HexColor::Hex6(value).to_string())
    }
}
impl Into<Color32> for &HexColor {
    fn into(self) -> Color32 {
        Color32::from_hex(&self.0).unwrap_or_default()
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Default, AsRefStr)]
pub enum CardKind {
    #[default]
    Unit,
    House,
}

#[derive(
    Clone,
    Copy,
    Debug,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Hash,
    Default,
    AsRefStr,
    EnumIter,
    Display,
)]
pub enum MagicType {
    #[default]
    Ability,
    Status,
}
