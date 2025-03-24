mod battle;
mod daily_updater;
mod global_data;
mod global_settings;
mod incubator;
mod inflating_number;
mod r#match;
mod nodes;
mod nodes_table;
mod player;
mod sync;

use std::str::FromStr;

use battle::*;
use glam::vec2;
use global_data::*;
use global_settings::*;
use incubator::*;
use inflating_number::*;
use itertools::Itertools;
use log::{debug, error, info};
use nodes::*;
use nodes_table::*;
use player::*;
use r#match::*;
use schema::*;
use spacetimedb::{reducer, table, Identity, ReducerContext, SpacetimeType, Table, Timestamp};
use std::collections::{HashMap, HashSet};

pub fn next_id(ctx: &ReducerContext) -> u64 {
    GlobalData::next_id(ctx)
}

const ADMIN_IDENTITY_HEX: &str = "c2006040747a1f04c2cebab8453bcf8b06c18e17f09e34ff20fd7883e748ca8e";
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
    All {
        name: "all".into(),
        id: ID_ALL,
        ..default()
    }
    .insert_self(ctx);
    Incubator {
        id: ID_INCUBATOR,
        name: "incubator".into(),
        ..default()
    }
    .insert_self(ctx);
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
