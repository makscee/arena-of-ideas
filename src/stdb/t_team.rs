// THIS FILE IS AUTOMATICALLY GENERATED BY SPACETIMEDB. EDITS TO THIS FILE
// WILL NOT BE SAVED. MODIFY TABLES IN RUST INSTEAD.

#![allow(unused_imports)]
use super::fused_unit::FusedUnit;
use super::team_pool::TeamPool;
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
pub struct TTeam {
    pub id: u64,
    pub name: String,
    pub owner: u64,
    pub units: Vec<FusedUnit>,
    pub pool: TeamPool,
}

impl TableType for TTeam {
    const TABLE_NAME: &'static str = "TTeam";
    type ReducerEvent = super::ReducerEvent;
}

impl TableWithPrimaryKey for TTeam {
    type PrimaryKey = u64;
    fn primary_key(&self) -> &Self::PrimaryKey {
        &self.id
    }
}

impl TTeam {
    #[allow(unused)]
    pub fn filter_by_id(id: u64) -> TableIter<Self> {
        Self::filter(|row| row.id == id)
    }
    #[allow(unused)]
    pub fn find_by_id(id: u64) -> Option<Self> {
        Self::find(|row| row.id == id)
    }
    #[allow(unused)]
    pub fn filter_by_name(name: String) -> TableIter<Self> {
        Self::filter(|row| row.name == name)
    }
    #[allow(unused)]
    pub fn filter_by_owner(owner: u64) -> TableIter<Self> {
        Self::filter(|row| row.owner == owner)
    }
}
