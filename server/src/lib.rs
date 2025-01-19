mod battle;
mod daily_updater;
mod global_data;
mod global_settings;
mod inflating_number;
mod nodes;
mod nodes_table;
mod player;
mod player_tag;
mod sync;
mod wallet;

use std::str::FromStr;

use battle::*;
use glam::vec2;
use global_data::*;
use global_settings::*;
use inflating_number::*;
use itertools::Itertools;
use nodes::*;
use nodes_table::*;
use player::*;
use player_tag::*;
use rand::{distributions::Alphanumeric, seq::IteratorRandom, Rng};
use schema::*;
use spacetimedb::{
    eprintln, println, reducer, table, Identity, ReducerContext, SpacetimeType, Table, Timestamp,
};
use wallet::*;

pub fn next_id(ctx: &ReducerContext) -> u64 {
    GlobalData::next_id(ctx)
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

#[reducer(init)]
fn init(ctx: &ReducerContext) -> Result<(), String> {
    GlobalData::init(ctx);
    Ok(())
}

#[reducer]
fn cleanup(ctx: &ReducerContext) -> Result<(), String> {
    ctx.is_admin()?;
    TPlayer::cleanup(ctx);
    Ok(())
}
