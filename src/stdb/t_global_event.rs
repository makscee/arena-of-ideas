// THIS FILE IS AUTOMATICALLY GENERATED BY SPACETIMEDB. EDITS TO THIS FILE
// WILL NOT BE SAVED. MODIFY TABLES IN RUST INSTEAD.

#![allow(unused_imports)]
use super::global_event::GlobalEvent;
use spacetimedb_sdk::{
    anyhow::{anyhow, Result},
    identity::Identity,
    reducer::{Reducer, ReducerCallbackId, Status},
    sats::{de::Deserialize, ser::Serialize},
    spacetimedb_lib,
    table::{TableIter, TableType, TableWithPrimaryKey},
    Address, ScheduleAt,
};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct TGlobalEvent {
    pub id: u64,
    pub owner: u64,
    pub event: GlobalEvent,
    pub ts: u64,
}

impl TableType for TGlobalEvent {
    const TABLE_NAME: &'static str = "TGlobalEvent";
    type ReducerEvent = super::ReducerEvent;
}

impl TableWithPrimaryKey for TGlobalEvent {
    type PrimaryKey = u64;
    fn primary_key(&self) -> &Self::PrimaryKey {
        &self.id
    }
}

impl TGlobalEvent {
    #[allow(unused)]
    pub fn filter_by_id(id: u64) -> TableIter<Self> {
        Self::filter(|row| row.id == id)
    }
    #[allow(unused)]
    pub fn find_by_id(id: u64) -> Option<Self> {
        Self::find(|row| row.id == id)
    }
    #[allow(unused)]
    pub fn filter_by_owner(owner: u64) -> TableIter<Self> {
        Self::filter(|row| row.owner == owner)
    }
    #[allow(unused)]
    pub fn filter_by_ts(ts: u64) -> TableIter<Self> {
        Self::filter(|row| row.ts == ts)
    }
}