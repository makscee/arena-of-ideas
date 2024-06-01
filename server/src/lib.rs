mod base_unit;
mod fused_unit;
mod global_data;
mod global_settings;
mod representation;
mod run;
mod user;

use anyhow::Context;
pub use fused_unit::*;
pub use global_data::*;
pub use global_settings::*;
pub use spacetimedb::SpacetimeType;
pub use spacetimedb::{spacetimedb, Identity, ReducerContext};
pub use user::*;

trait StrContext<T> {
    fn context_str(self, str: &'static str) -> Result<T, String>;
}

impl<T> StrContext<T> for Option<T> {
    fn context_str(self, str: &'static str) -> Result<T, String> {
        self.context(str).map_err(|e| e.to_string())
    }
}
