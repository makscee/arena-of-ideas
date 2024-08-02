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
pub struct RunStartRankedArgs {}

impl Reducer for RunStartRankedArgs {
    const REDUCER_NAME: &'static str = "run_start_ranked";
}

#[allow(unused)]
pub fn run_start_ranked() {
    RunStartRankedArgs {}.invoke();
}

#[allow(unused)]
pub fn on_run_start_ranked(
    mut __callback: impl FnMut(&Identity, Option<Address>, &Status) + Send + 'static,
) -> ReducerCallbackId<RunStartRankedArgs> {
    RunStartRankedArgs::on_reducer(move |__identity, __addr, __status, __args| {
        let RunStartRankedArgs {} = __args;
        __callback(__identity, __addr, __status);
    })
}

#[allow(unused)]
pub fn once_on_run_start_ranked(
    __callback: impl FnOnce(&Identity, Option<Address>, &Status) + Send + 'static,
) -> ReducerCallbackId<RunStartRankedArgs> {
    RunStartRankedArgs::once_on_reducer(move |__identity, __addr, __status, __args| {
        let RunStartRankedArgs {} = __args;
        __callback(__identity, __addr, __status);
    })
}

#[allow(unused)]
pub fn remove_on_run_start_ranked(id: ReducerCallbackId<RunStartRankedArgs>) {
    RunStartRankedArgs::remove_on_reducer(id);
}