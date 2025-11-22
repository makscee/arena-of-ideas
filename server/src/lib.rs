mod admin;
mod battle_table;
mod content;
mod context;
mod daily_updater;
mod global_data;
mod global_settings;
mod r#match;
mod nodes;
pub mod nodes_table;
mod player;

use std::str::FromStr;

pub use admin::*;
pub use context::*;
use global_data::*;
use global_settings::GlobalSettings;

use itertools::Itertools;
use log::{debug, error, info};
use nodes::*;
pub use nodes_table::*;
use player::*;

use schema::*;
use spacetimedb::Table;
use spacetimedb::{Identity, ReducerContext, SpacetimeType, Timestamp, reducer, table};
use std::collections::HashMap;

use crate::global_data::global_data as _;
use crate::global_settings::global_settings as _;

pub fn next_id(ctx: &ReducerContext) -> u64 {
    GlobalData::next_id(ctx)
}

#[reducer(init)]
pub fn init(ctx: &ReducerContext) -> Result<(), String> {
    if ctx.db.global_data().count() == 0 {
        GlobalData::init(ctx);
    }
    if ctx.db.global_settings().count() == 0 {
        GlobalSettings::default().replace(ctx);
    }
    if ctx.db.nodes_world().id().find(ID_ARENA).is_none() {
        NArena {
            id: ID_ARENA,
            owner: ID_ARENA,
            last_floor: 0,
            ..Default::default()
        }
        .insert(&ctx.as_context());
    }

    Ok(())
}

trait CtxExt {
    fn global_settings(&self) -> GlobalSettings;
    fn next_id(&self) -> u64;
}

impl CtxExt for ServerContext<'_> {
    fn global_settings(&self) -> GlobalSettings {
        self.rctx().db.global_data().count();
        GlobalSettings::get(self.rctx())
    }
    fn next_id(&self) -> u64 {
        GlobalData::next_id(self.rctx())
    }
}
