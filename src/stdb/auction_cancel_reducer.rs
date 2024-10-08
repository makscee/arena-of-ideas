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
pub struct AuctionCancelArgs {
    pub item_id: u64,
}

impl Reducer for AuctionCancelArgs {
    const REDUCER_NAME: &'static str = "auction_cancel";
}

#[allow(unused)]
pub fn auction_cancel(item_id: u64) {
    AuctionCancelArgs { item_id }.invoke();
}

#[allow(unused)]
pub fn on_auction_cancel(
    mut __callback: impl FnMut(&Identity, Option<Address>, &Status, &u64) + Send + 'static,
) -> ReducerCallbackId<AuctionCancelArgs> {
    AuctionCancelArgs::on_reducer(move |__identity, __addr, __status, __args| {
        let AuctionCancelArgs { item_id } = __args;
        __callback(__identity, __addr, __status, item_id);
    })
}

#[allow(unused)]
pub fn once_on_auction_cancel(
    __callback: impl FnOnce(&Identity, Option<Address>, &Status, &u64) + Send + 'static,
) -> ReducerCallbackId<AuctionCancelArgs> {
    AuctionCancelArgs::once_on_reducer(move |__identity, __addr, __status, __args| {
        let AuctionCancelArgs { item_id } = __args;
        __callback(__identity, __addr, __status, item_id);
    })
}

#[allow(unused)]
pub fn remove_on_auction_cancel(id: ReducerCallbackId<AuctionCancelArgs>) {
    AuctionCancelArgs::remove_on_reducer(id);
}
