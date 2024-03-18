mod ability;
mod arena_pool;
mod arena_run;
mod global_data;
mod house;
mod status;
mod summon;
mod unit;
mod user;
mod user_access;
mod vfx;

pub use ability::*;
pub use anyhow::Context;
pub use arena_pool::*;
pub use arena_run::*;
pub use global_data::GlobalData;
pub use house::*;
pub use spacetimedb::SpacetimeType;
pub use spacetimedb::{spacetimedb, Identity, ReducerContext};
pub use status::*;
pub use std::str::FromStr;
pub use summon::*;
pub use unit::*;
pub use user::*;
pub use user_access::*;
pub use vfx::*;

#[spacetimedb(init)]
fn init_user_access() -> Result<(), String> {
    UserAccess::init()?;
    GlobalData::init()?;
    Ok(())
}

trait StrContext<T> {
    fn context_str(self, str: &'static str) -> Result<T, String>;
}

impl<T> StrContext<T> for Option<T> {
    fn context_str(self, str: &'static str) -> Result<T, String> {
        self.context(str).map_err(|e| e.to_string())
    }
}
