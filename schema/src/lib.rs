pub mod error;
pub mod macro_fn;
mod nodes;
pub mod var_name;
pub mod var_value;

use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, Display, EnumIter, EnumString};
