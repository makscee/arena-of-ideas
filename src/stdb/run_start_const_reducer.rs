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
    Address, ScheduleAt,
};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct RunStartConstArgs {}

impl Reducer for RunStartConstArgs {
    const REDUCER_NAME: &'static str = "run_start_const";
}

#[allow(unused)]
pub fn run_start_const() {
    RunStartConstArgs {}.invoke();
}

#[allow(unused)]
pub fn on_run_start_const(
    mut __callback: impl FnMut(&Identity, Option<Address>, &Status) + Send + 'static,
) -> ReducerCallbackId<RunStartConstArgs> {
    RunStartConstArgs::on_reducer(move |__identity, __addr, __status, __args| {
        let RunStartConstArgs {} = __args;
        __callback(__identity, __addr, __status);
    })
}

#[allow(unused)]
pub fn once_on_run_start_const(
    __callback: impl FnOnce(&Identity, Option<Address>, &Status) + Send + 'static,
) -> ReducerCallbackId<RunStartConstArgs> {
    RunStartConstArgs::once_on_reducer(move |__identity, __addr, __status, __args| {
        let RunStartConstArgs {} = __args;
        __callback(__identity, __addr, __status);
    })
}

#[allow(unused)]
pub fn remove_on_run_start_const(id: ReducerCallbackId<RunStartConstArgs>) {
    RunStartConstArgs::remove_on_reducer(id);
}
