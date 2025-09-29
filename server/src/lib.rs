mod admin;
mod content;
mod context;
mod daily_updater;
mod global_data;
mod global_settings;
mod inflating_number;
mod r#match;
mod nodes;
pub mod nodes_table;
mod player;
mod votes;

use std::str::FromStr;

pub use context::*;
use global_data::*;
use global_settings::*;
use itertools::Itertools;
use log::{debug, error, info};
use nodes::*;
pub use nodes_table::*;
use player::*;

use schema::*;
use spacetimedb::{Identity, ReducerContext, SpacetimeType, Table, Timestamp, reducer, table};
use std::collections::{HashMap, HashSet};

// Include generated server nodes
include!(concat!(env!("OUT_DIR"), "/server_nodes.rs"));

pub fn next_id(ctx: &ReducerContext) -> u64 {
    GlobalData::next_id(ctx)
}

#[reducer(init)]
fn init(ctx: &ReducerContext) -> Result<(), String> {
    GlobalData::init(ctx);
    GlobalSettings::default().replace(ctx);
    NArena {
        id: Some(ID_ARENA),
        ..Default::default()
    }
    .insert(ctx);
    Ok(())
}

trait CtxExt {
    fn global_settings(&self) -> GlobalSettings;
    fn next_id(&self) -> u64;
}

impl CtxExt for ReducerContext {
    fn global_settings(&self) -> GlobalSettings {
        GlobalSettings::get(self)
    }
    fn next_id(&self) -> u64 {
        GlobalData::next_id(self)
    }
}
