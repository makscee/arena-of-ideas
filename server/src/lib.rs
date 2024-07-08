mod ability;
mod base_unit;
mod fused_unit;
mod global_data;
mod global_settings;
mod house;
mod representation;
mod run;
mod status;
mod sync;
mod team;
mod user;

use anyhow::Context;
pub use fused_unit::*;
pub use global_data::*;
pub use global_settings::*;
pub use spacetimedb::SpacetimeType;
pub use spacetimedb::{spacetimedb, Identity, ReducerContext};
pub use team::*;
pub use user::*;

pub type GID = u64;

trait StrContext<T> {
    fn context_str(self, str: &'static str) -> Result<T, String>;
}

impl<T> StrContext<T> for Option<T> {
    fn context_str(self, str: &'static str) -> Result<T, String> {
        self.context(str).map_err(|e| e.to_string())
    }
}

#[spacetimedb(init)]
fn init() -> Result<(), String> {
    GlobalData::init()?;
    Ok(())
}

pub fn next_id() -> GID {
    GlobalData::next_id()
}
