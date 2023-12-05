// THIS FILE IS AUTOMATICALLY GENERATED BY SPACETIMEDB. EDITS TO THIS FILE
// WILL NOT BE SAVED. MODIFY TABLES IN RUST INSTEAD.

#[allow(unused)]
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
pub struct Ladder {
    pub id: u64,
    pub owner: Identity,
    pub levels: Vec<String>,
}

impl TableType for Ladder {
    const TABLE_NAME: &'static str = "Ladder";
    type ReducerEvent = super::ReducerEvent;
}

impl TableWithPrimaryKey for Ladder {
    type PrimaryKey = u64;
    fn primary_key(&self) -> &Self::PrimaryKey {
        &self.id
    }
}

impl Ladder {
    #[allow(unused)]
    pub fn filter_by_id(id: u64) -> Option<Self> {
        Self::find(|row| row.id == id)
    }
    #[allow(unused)]
    pub fn filter_by_owner(owner: Identity) -> TableIter<Self> {
        Self::filter(|row| row.owner == owner)
    }
    #[allow(unused)]
    pub fn filter_by_levels(levels: Vec<String>) -> TableIter<Self> {
        Self::filter(|row| row.levels == levels)
    }
}