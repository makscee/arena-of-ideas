// THIS FILE IS AUTOMATICALLY GENERATED BY SPACETIMEDB. EDITS TO THIS FILE
// WILL NOT BE SAVED. MODIFY TABLES IN RUST INSTEAD.

use super::table_unit::TableUnit;
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
pub struct UploadUnitsArgs {
    pub units: Vec<TableUnit>,
}

impl Reducer for UploadUnitsArgs {
    const REDUCER_NAME: &'static str = "upload_units";
}

#[allow(unused)]
pub fn upload_units(units: Vec<TableUnit>) {
    UploadUnitsArgs { units }.invoke();
}

#[allow(unused)]
pub fn on_upload_units(
    mut __callback: impl FnMut(&Identity, Option<Address>, &Status, &Vec<TableUnit>) + Send + 'static,
) -> ReducerCallbackId<UploadUnitsArgs> {
    UploadUnitsArgs::on_reducer(move |__identity, __addr, __status, __args| {
        let UploadUnitsArgs { units } = __args;
        __callback(__identity, __addr, __status, units);
    })
}

#[allow(unused)]
pub fn once_on_upload_units(
    __callback: impl FnOnce(&Identity, Option<Address>, &Status, &Vec<TableUnit>) + Send + 'static,
) -> ReducerCallbackId<UploadUnitsArgs> {
    UploadUnitsArgs::once_on_reducer(move |__identity, __addr, __status, __args| {
        let UploadUnitsArgs { units } = __args;
        __callback(__identity, __addr, __status, units);
    })
}

#[allow(unused)]
pub fn remove_on_upload_units(id: ReducerCallbackId<UploadUnitsArgs>) {
    UploadUnitsArgs::remove_on_reducer(id);
}