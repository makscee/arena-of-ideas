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
mod starting_hero;
mod status;
mod sync;
mod team;
mod trade;
mod user;
mod wallet;

use std::str::FromStr;

use anyhow::Context;
pub use arena::*;
pub use arena_leaderboard::*;
pub use arena_pool::*;
pub use base_unit::*;
pub use battle::*;
pub use fused_unit::*;
pub use global_data::*;
pub use global_settings::*;
pub use item::*;
pub use itertools::Itertools;
pub use meta_shop::*;
pub use rand::{distributions::Alphanumeric, seq::IteratorRandom, thread_rng, Rng};
pub use spacetimedb::{
    schedule, spacetimedb, Identity, ReducerContext, SpacetimeType, TableType, Timestamp,
};
pub use starting_hero::*;
pub use team::*;
pub use trade::*;
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

const ADMIN_IDENTITY_HEX: &str = "6617138da5d77d4289f74b2fff59e9aa1a7b87292f23908eccaa3510b606729e";
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
