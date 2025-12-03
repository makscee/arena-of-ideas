mod admin;
mod battle_table;
mod content;
mod context;
mod creation_phases;
mod daily_updater;
mod global_data;
mod global_settings;
mod link_ext;
mod r#match;
mod nodes;
pub mod nodes_table;
mod player;

use std::str::FromStr;

pub use admin::*;
pub use context::*;
pub use creation_phases::*;
use global_data::*;
use global_settings::GlobalSettings;
pub use link_ext::{ServerMultipleLinkLoad, ServerSingleLinkLoad};

use itertools::Itertools;
use log::{debug, error, info};
use nodes::*;
use nodes_table::*;
use player::*;

use schema::*;
use serde::de::DeserializeOwned;
use spacetimedb::Table;
use spacetimedb::{Identity, ReducerContext, SpacetimeType, Timestamp, reducer, table};
use std::collections::HashMap;

use crate::global_data::global_data as _;
use crate::global_settings::global_settings as _;

pub fn next_id(ctx: &ServerContext<'_>) -> u64 {
    GlobalData::next_id(ctx)
}

#[reducer(init)]
pub fn init(ctx: &ReducerContext) -> Result<(), String> {
    let ctx = &ctx.as_context();
    if ctx.rctx().db.global_data().count() == 0 {
        GlobalData::init(ctx);
    }
    if ctx.rctx().db.global_settings().count() == 0 {
        GlobalSettings::default().replace(ctx);
    }
    if ctx.rctx().db.nodes_world().id().find(ID_ARENA).is_none() {
        NArena {
            id: ID_ARENA,
            owner: ID_ARENA,
            last_floor: 0,
            ..Default::default()
        }
        .insert(ctx);
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
        GlobalSettings::get(self)
    }
    fn next_id(&self) -> u64 {
        GlobalData::next_id(self)
    }
}
