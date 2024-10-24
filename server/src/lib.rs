mod ability;
mod arena;
mod arena_leaderboard;
mod arena_pool;
mod auction;
mod base_unit;
mod battle;
mod daily_state;
mod daily_updater;
mod fused_unit;
mod global_data;
mod global_settings;
mod house;
mod inflating_number;
mod items;
mod meta_shop;
mod quest;
mod representation;
mod status;
mod sync;
mod team;
mod trade;
mod unit_balance;
mod user;
mod wallet;

use std::str::FromStr;

use anyhow::Context;
pub use arena::*;
pub use arena_leaderboard::*;
pub use arena_pool::*;
pub use base_unit::*;
pub use battle::*;
pub use daily_state::*;
use daily_updater::daily_timer_init;
pub use fused_unit::*;
pub use global_data::*;
pub use global_settings::*;
pub use inflating_number::*;
pub use items::*;
pub use itertools::Itertools;
pub use meta_shop::*;
pub use quest::*;
pub use rand::{distributions::Alphanumeric, seq::IteratorRandom, Rng};
pub use spacetimedb::rng;
pub use spacetimedb::{eprintln, println};
pub use spacetimedb::{spacetimedb, Identity, ReducerContext, SpacetimeType, TableType, Timestamp};
pub use team::*;
pub use trade::*;
pub use unit_balance::*;
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

#[derive(SpacetimeType, Clone, Default)]
pub enum GameMode {
    #[default]
    ArenaNormal,
    ArenaRanked,
    ArenaConst(String),
}

impl PartialEq for GameMode {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
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

#[inline]
pub fn default<T: Default>() -> T {
    Default::default()
}
