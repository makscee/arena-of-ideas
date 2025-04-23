mod action;
mod error;
mod event;
mod expression;
mod fusion;
mod inject;
mod macro_fn;
#[allow(dead_code)]
mod nodes;
mod packed_nodes;
mod painter_action;
mod reaction;
mod trigger;
mod var_name;
mod var_value;

use std::fmt::Display;

pub use action::*;
use ecolor::Color32;
pub use error::*;
pub use event::*;
pub use expression::*;
pub use fusion::*;
pub use inject::*;
pub use macro_fn::*;
pub use packed_nodes::*;
pub use painter_action::*;
pub use reaction::*;
use ron::ser::{to_string_pretty, PrettyConfig};
pub use trigger::*;
pub use var_name::*;
pub use var_value::*;

pub use glam::Vec2;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use strum_macros::{AsRefStr, Display, EnumIter, EnumString};
pub use utils::*;

pub const ID_CORE: u64 = 1;
pub const ID_PLAYERS: u64 = 2;
pub const ID_ARENA: u64 = 3;

pub const NODE_CONTAINERS: [u64; 3] = [ID_CORE, ID_PLAYERS, ID_ARENA];

pub trait StringData: Sized {
    fn inject_data(&mut self, data: &str) -> Result<(), ExpressionError>;
    fn get_data(&self) -> String;
}
impl<T> StringData for T
where
    T: Serialize + DeserializeOwned,
{
    fn inject_data(&mut self, data: &str) -> Result<(), ExpressionError> {
        match ron::from_str(data) {
            Ok(v) => {
                *self = v;
                Ok(())
            }
            Err(e) => Err(format!("Deserialize error: {e}").into()),
        }
    }
    fn get_data(&self) -> String {
        to_string_pretty(self, PrettyConfig::new().depth_limit(1)).unwrap()
    }
}

pub type LinkMany<T> = Vec<T>;
pub type LinkOne<T> = Option<T>;

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
