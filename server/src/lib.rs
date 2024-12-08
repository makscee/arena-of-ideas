pub mod arena_leaderboard;
pub mod arena_pool;
pub mod daily_state;
pub mod daily_updater;
pub mod global_data;
pub mod global_settings;
pub mod inflating_number;
pub mod player;
pub mod player_stats;
pub mod player_tag;
pub mod quest;
pub mod wallet;

use std::str::FromStr;

use anyhow::Context;
pub use arena_leaderboard::*;
pub use arena_pool::*;
pub use daily_state::*;
pub use global_data::*;
pub use global_settings::*;
pub use inflating_number::*;
pub use itertools::Itertools;
pub use player::*;
pub use player_tag::*;
pub use quest::*;
pub use rand::{distributions::Alphanumeric, seq::IteratorRandom, Rng};
pub use spacetimedb::{
    eprintln, println, reducer, table, Identity, ReducerContext, SpacetimeType, Table, Timestamp,
};
use utils::*;
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

pub fn next_id(ctx: &ReducerContext) -> u64 {
    GlobalData::next_id(ctx)
}

#[derive(SpacetimeType, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum GameMode {
    #[default]
    ArenaNormal = 0,
    ArenaRanked = 1,
    ArenaConst = 2,
}

const ADMIN_IDENTITY_HEX: &str = "c2000d3d36c3162dd302f78b29d2e3b78af2e0d9310cbe8fe9d75af5e9c393d0";
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

#[spacetimedb::reducer(init)]
fn init(ctx: &ReducerContext) -> Result<(), String> {
    GlobalData::init(ctx);
    Ok(())
}

#[spacetimedb::reducer]
fn cleanup(ctx: &ReducerContext) -> Result<(), String> {
    ctx.is_admin()?;
    TPlayer::cleanup(ctx);
    Ok(())
}
