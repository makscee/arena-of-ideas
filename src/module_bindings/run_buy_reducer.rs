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
pub struct RunBuyArgs {
    pub id: u64,
}

impl Reducer for RunBuyArgs {
    const REDUCER_NAME: &'static str = "run_buy";
}

#[allow(unused)]
pub fn run_buy(id: u64) {
    RunBuyArgs { id }.invoke();
}

#[allow(unused)]
pub fn on_run_buy(
    mut __callback: impl FnMut(&Identity, Option<Address>, &Status, &u64) + Send + 'static,
) -> ReducerCallbackId<RunBuyArgs> {
    RunBuyArgs::on_reducer(move |__identity, __addr, __status, __args| {
        let RunBuyArgs { id } = __args;
        __callback(__identity, __addr, __status, id);
    })
}

#[allow(unused)]
pub fn once_on_run_buy(
    __callback: impl FnOnce(&Identity, Option<Address>, &Status, &u64) + Send + 'static,
) -> ReducerCallbackId<RunBuyArgs> {
    RunBuyArgs::once_on_reducer(move |__identity, __addr, __status, __args| {
        let RunBuyArgs { id } = __args;
        __callback(__identity, __addr, __status, id);
    })
}

#[allow(unused)]
pub fn remove_on_run_buy(id: ReducerCallbackId<RunBuyArgs>) {
    RunBuyArgs::remove_on_reducer(id);
}
