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
pub struct GiveCreditsArgs {}

impl Reducer for GiveCreditsArgs {
    const REDUCER_NAME: &'static str = "give_credits";
}

#[allow(unused)]
pub fn give_credits() {
    GiveCreditsArgs {}.invoke();
}

#[allow(unused)]
pub fn on_give_credits(
    mut __callback: impl FnMut(&Identity, Option<Address>, &Status) + Send + 'static,
) -> ReducerCallbackId<GiveCreditsArgs> {
    GiveCreditsArgs::on_reducer(move |__identity, __addr, __status, __args| {
        let GiveCreditsArgs {} = __args;
        __callback(__identity, __addr, __status);
    })
}

#[allow(unused)]
pub fn once_on_give_credits(
    __callback: impl FnOnce(&Identity, Option<Address>, &Status) + Send + 'static,
) -> ReducerCallbackId<GiveCreditsArgs> {
    GiveCreditsArgs::once_on_reducer(move |__identity, __addr, __status, __args| {
        let GiveCreditsArgs {} = __args;
        __callback(__identity, __addr, __status);
    })
}

#[allow(unused)]
pub fn remove_on_give_credits(id: ReducerCallbackId<GiveCreditsArgs>) {
    GiveCreditsArgs::remove_on_reducer(id);
}