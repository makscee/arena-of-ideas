mod ability;
mod arena;
mod arena_leaderboard;
mod arena_pool;
mod base_unit;
mod battle;
mod fused_unit;
mod global_data;
mod global_settings;
mod house;
mod item;
mod meta_shop;
mod representation;
mod status;
mod sync;
mod team;
mod user;
mod wallet;

use std::str::FromStr;

use anyhow::Context;
pub use arena::*;
pub use arena_leaderboard::*;
pub use arena_pool::*;
pub use battle::*;
pub use fused_unit::*;
pub use global_data::*;
pub use global_settings::*;
pub use item::*;
pub use meta_shop::*;
pub use spacetimedb::SpacetimeType;
pub use spacetimedb::{spacetimedb, Identity, ReducerContext};
pub use team::*;
pub use user::*;
pub use wallet::*;

trait StrContext<T> {
    fn context_str(self, str: &'static str) -> Result<T, String>;
    fn with_context_str<F>(self, f: F) -> Result<T, String>
    where
        F: FnOnce() -> String;
}

impl<T> StrContext<T> for Option<T> {
    fn context_str(self, str: &'static str) -> Result<T, String> {
        self.context(str).map_err(|e| e.to_string())
    }

    fn with_context_str<F>(self, f: F) -> Result<T, String>
    where
        F: FnOnce() -> String,
    {
        self.with_context(f).map_err(|e| e.to_string())
    }
}

pub fn next_id() -> u64 {
    GlobalData::next_id()
}

#[derive(SpacetimeType, Clone, PartialEq, Eq)]
pub enum GameMode {
    ArenaNormal,
    ArenaRanked,
    ArenaConst(String),
}

const ADMIN_IDENTITY_HEX: &str = "ad22b9dc867768c48531281bab2d5e0be1f915c4e46d107547bf624fb6dbf26c";
pub fn is_admin(identity: &Identity) -> Result<bool, String> {
    Ok(Identity::from_str(ADMIN_IDENTITY_HEX)
        .map_err(|e| e.to_string())?
        .eq(identity))
}

pub trait AdminCheck {
    fn is_admin(self) -> Result<(), String>;
}

impl AdminCheck for &ReducerContext {
    fn is_admin(self) -> Result<(), String> {
        if is_admin(&self.sender)? {
            Ok(())
        } else {
            Err("Need admin access".to_owned())
        }
    }
}

#[spacetimedb(init)]
fn init() -> Result<(), String> {
    GlobalData::init()?;
    Ok(())
}
