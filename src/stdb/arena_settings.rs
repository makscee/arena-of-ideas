// THIS FILE IS AUTOMATICALLY GENERATED BY SPACETIMEDB. EDITS TO THIS FILE
// WILL NOT BE SAVED. MODIFY TABLES IN RUST INSTEAD.

#![allow(unused_imports)]
use spacetimedb_sdk::{
    anyhow::{anyhow, Result},
    identity::Identity,
    reducer::{Reducer, ReducerCallbackId, Status},
    sats::{de::Deserialize, ser::Serialize},
    spacetimedb_lib,
    table::{TableIter, TableType, TableWithPrimaryKey},
    Address,
};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct ArenaSettings {
    pub slots_min: u32,
    pub slots_max: u32,
    pub slots_per_round: f32,
    pub g_start: i32,
    pub g_income_min: i32,
    pub g_income_max: i32,
    pub g_income_per_round: i32,
    pub price_reroll: i32,
    pub sell_discount: i32,
    pub stack_discount: i32,
    pub team_slots: u32,
}