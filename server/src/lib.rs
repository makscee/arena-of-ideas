pub mod daily_updater;
pub mod global_data;
pub mod global_settings;
pub mod inflating_number;
mod nodes;
pub mod player;
pub mod player_tag;
pub mod wallet;

use std::str::FromStr;

pub use global_data::*;
pub use global_settings::*;
pub use inflating_number::*;
pub use itertools::Itertools;
use nodes_server::{House, Node};
pub use player::*;
pub use player_tag::*;
use rand::{distributions::Alphanumeric, seq::IteratorRandom, Rng};
use schema::ExpressionError;
use spacetimedb::{
    eprintln, println, reducer, table, Identity, ReducerContext, SpacetimeType, Table, Timestamp,
};
pub use wallet::*;

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

fn f() {
    let h = House {
        name: String::new(),
        color: None,
        abilities: Vec::new(),
    };
    let s = h.get_data();
}
