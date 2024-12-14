pub mod effect;
pub mod error;
pub mod expression;
pub mod macro_fn;
mod material;
mod nodes;
mod trigger;
pub mod var_name;
pub mod var_value;

pub use material::*;
use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, Display, EnumIter, EnumString};
pub use trigger::*;
use utils::*;
