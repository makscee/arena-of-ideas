// THIS FILE IS AUTOMATICALLY GENERATED BY SPACETIMEDB. EDITS TO THIS FILE
// WILL NOT BE SAVED. MODIFY TABLES IN RUST INSTEAD.

use super::summon::Summon;
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
pub struct SyncSummonsArgs {
    pub summons: Vec<Summon>,
}

impl Reducer for SyncSummonsArgs {
    const REDUCER_NAME: &'static str = "sync_summons";
}

#[allow(unused)]
pub fn sync_summons(summons: Vec<Summon>) {
    SyncSummonsArgs { summons }.invoke();
}

#[allow(unused)]
pub fn on_sync_summons(
    mut __callback: impl FnMut(&Identity, Option<Address>, &Status, &Vec<Summon>) + Send + 'static,
) -> ReducerCallbackId<SyncSummonsArgs> {
    SyncSummonsArgs::on_reducer(move |__identity, __addr, __status, __args| {
        let SyncSummonsArgs { summons } = __args;
        __callback(__identity, __addr, __status, summons);
    })
}

#[allow(unused)]
pub fn once_on_sync_summons(
    __callback: impl FnOnce(&Identity, Option<Address>, &Status, &Vec<Summon>) + Send + 'static,
) -> ReducerCallbackId<SyncSummonsArgs> {
    SyncSummonsArgs::once_on_reducer(move |__identity, __addr, __status, __args| {
        let SyncSummonsArgs { summons } = __args;
        __callback(__identity, __addr, __status, summons);
    })
}

#[allow(unused)]
pub fn remove_on_sync_summons(id: ReducerCallbackId<SyncSummonsArgs>) {
    SyncSummonsArgs::remove_on_reducer(id);
}
