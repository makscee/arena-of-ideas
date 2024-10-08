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
pub struct StackShopArgs {
    pub source: u8,
    pub target: u8,
}

impl Reducer for StackShopArgs {
    const REDUCER_NAME: &'static str = "stack_shop";
}

#[allow(unused)]
pub fn stack_shop(source: u8, target: u8) {
    StackShopArgs { source, target }.invoke();
}

#[allow(unused)]
pub fn on_stack_shop(
    mut __callback: impl FnMut(&Identity, Option<Address>, &Status, &u8, &u8) + Send + 'static,
) -> ReducerCallbackId<StackShopArgs> {
    StackShopArgs::on_reducer(move |__identity, __addr, __status, __args| {
        let StackShopArgs { source, target } = __args;
        __callback(__identity, __addr, __status, source, target);
    })
}

#[allow(unused)]
pub fn once_on_stack_shop(
    __callback: impl FnOnce(&Identity, Option<Address>, &Status, &u8, &u8) + Send + 'static,
) -> ReducerCallbackId<StackShopArgs> {
    StackShopArgs::once_on_reducer(move |__identity, __addr, __status, __args| {
        let StackShopArgs { source, target } = __args;
        __callback(__identity, __addr, __status, source, target);
    })
}

#[allow(unused)]
pub fn remove_on_stack_shop(id: ReducerCallbackId<StackShopArgs>) {
    StackShopArgs::remove_on_reducer(id);
}
