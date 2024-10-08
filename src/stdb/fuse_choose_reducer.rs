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
pub struct FuseChooseArgs {
    pub slot: u8,
}

impl Reducer for FuseChooseArgs {
    const REDUCER_NAME: &'static str = "fuse_choose";
}

#[allow(unused)]
pub fn fuse_choose(slot: u8) {
    FuseChooseArgs { slot }.invoke();
}

#[allow(unused)]
pub fn on_fuse_choose(
    mut __callback: impl FnMut(&Identity, Option<Address>, &Status, &u8) + Send + 'static,
) -> ReducerCallbackId<FuseChooseArgs> {
    FuseChooseArgs::on_reducer(move |__identity, __addr, __status, __args| {
        let FuseChooseArgs { slot } = __args;
        __callback(__identity, __addr, __status, slot);
    })
}

#[allow(unused)]
pub fn once_on_fuse_choose(
    __callback: impl FnOnce(&Identity, Option<Address>, &Status, &u8) + Send + 'static,
) -> ReducerCallbackId<FuseChooseArgs> {
    FuseChooseArgs::once_on_reducer(move |__identity, __addr, __status, __args| {
        let FuseChooseArgs { slot } = __args;
        __callback(__identity, __addr, __status, slot);
    })
}

#[allow(unused)]
pub fn remove_on_fuse_choose(id: ReducerCallbackId<FuseChooseArgs>) {
    FuseChooseArgs::remove_on_reducer(id);
}
