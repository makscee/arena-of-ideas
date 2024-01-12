mod ability;
mod arena_pool;
mod arena_run;
mod global_data;
mod global_tower;
mod house;
mod status;
mod unit;
mod user;
mod user_access;
mod vfx;

pub use anyhow::Context;
pub use arena_pool::*;
pub use arena_run::*;
pub use global_data::GlobalData;
pub use global_tower::*;
pub use spacetimedb::SpacetimeType;
pub use spacetimedb::{spacetimedb, Identity, ReducerContext};
pub use std::str::FromStr;
pub use user::*;
pub use user_access::*;

#[spacetimedb(init)]
fn init_user_access() -> Result<(), String> {
    UserAccess::init()?;
    GlobalTower::init()?;
    GlobalData::init()?;
    Ok(())
}
