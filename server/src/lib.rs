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

#[macro_export]
macro_rules! node_kind_match {
    ($kind:expr, $code:expr) => {
        match $kind {
            NodeKind::None => {
                unreachable!()
            }
            NodeKind::NArena => {
                type NodeType = NArena;
                $code
            }
            NodeKind::NFloorPool => {
                type NodeType = NFloorPool;
                $code
            }
            NodeKind::NFloorBoss => {
                type NodeType = NFloorBoss;
                $code
            }
            NodeKind::NPlayer => {
                type NodeType = NPlayer;
                $code
            }
            NodeKind::NPlayerData => {
                type NodeType = NPlayerData;
                $code
            }
            NodeKind::NPlayerIdentity => {
                type NodeType = NPlayerIdentity;
                $code
            }
            NodeKind::NHouse => {
                type NodeType = NHouse;
                $code
            }
            NodeKind::NHouseColor => {
                type NodeType = NHouseColor;
                $code
            }
            NodeKind::NAbilityMagic => {
                type NodeType = NAbilityMagic;
                $code
            }
            NodeKind::NAbilityDescription => {
                type NodeType = NAbilityDescription;
                $code
            }
            NodeKind::NAbilityEffect => {
                type NodeType = NAbilityEffect;
                $code
            }
            NodeKind::NStatusMagic => {
                type NodeType = NStatusMagic;
                $code
            }
            NodeKind::NStatusDescription => {
                type NodeType = NStatusDescription;
                $code
            }
            NodeKind::NStatusBehavior => {
                type NodeType = NStatusBehavior;
                $code
            }
            NodeKind::NStatusRepresentation => {
                type NodeType = NStatusRepresentation;
                $code
            }
            NodeKind::NStatusState => {
                type NodeType = NStatusState;
                $code
            }
            NodeKind::NTeam => {
                type NodeType = NTeam;
                $code
            }
            NodeKind::NBattle => {
                type NodeType = NBattle;
                $code
            }
            NodeKind::NMatch => {
                type NodeType = NMatch;
                $code
            }
            NodeKind::NFusion => {
                type NodeType = NFusion;
                $code
            }
            NodeKind::NFusionSlot => {
                type NodeType = NFusionSlot;
                $code
            }
            NodeKind::NUnit => {
                type NodeType = NUnit;
                $code
            }
            NodeKind::NUnitDescription => {
                type NodeType = NUnitDescription;
                $code
            }
            NodeKind::NUnitStats => {
                type NodeType = NUnitStats;
                $code
            }
            NodeKind::NUnitState => {
                type NodeType = NUnitState;
                $code
            }
            NodeKind::NUnitBehavior => {
                type NodeType = NUnitBehavior;
                $code
            }
            NodeKind::NUnitRepresentation => {
                type NodeType = NUnitRepresentation;
                $code
            }
        }
    };
}

pub fn next_id(ctx: &ReducerContext) -> u64 {
    GlobalData::next_id(ctx)
}

#[reducer(init)]
fn init(ctx: &ReducerContext) -> Result<(), String> {
    GlobalData::init(ctx);
    GlobalSettings::default().replace(ctx);
    NArena {
        id: ID_ARENA,
        ..Default::default()
    }
    .insert(&ctx.as_context());
    Ok(())
}

trait CtxExt {
    fn global_settings(&self) -> GlobalSettings;
    fn next_id(&self) -> u64;
}

impl CtxExt for ServerContext<'_> {
    fn global_settings(&self) -> GlobalSettings {
        GlobalSettings::get(self.rctx())
    }
    fn next_id(&self) -> u64 {
        GlobalData::next_id(self.rctx())
    }
}
