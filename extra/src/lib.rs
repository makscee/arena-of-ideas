pub use bevy::prelude::Entity;
pub use itertools::Itertools;
pub use serde::{Deserialize, Serialize};
pub use std::error::Error;
pub use strum_macros::{AsRefStr, Display, EnumIter, EnumString};
pub use thiserror::Error;

pub mod effect;
pub mod event;
pub mod expression;
pub mod nodes;
pub mod trigger;
pub mod var_name;
pub mod var_value;

pub use effect::*;
pub use event::*;
pub use expression::*;
pub use trigger::*;
pub use var_name::*;
pub use var_value::*;
#[macro_use]
extern crate extra_macros;

#[inline]
pub fn default<T: Default>() -> T {
    Default::default()
}
