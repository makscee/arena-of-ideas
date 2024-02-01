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
pub struct LoginByIdentityArgs {}

impl Reducer for LoginByIdentityArgs {
    const REDUCER_NAME: &'static str = "login_by_identity";
}

#[allow(unused)]
pub fn login_by_identity() {
    LoginByIdentityArgs {}.invoke();
}

#[allow(unused)]
pub fn on_login_by_identity(
    mut __callback: impl FnMut(&Identity, Option<Address>, &Status) + Send + 'static,
) -> ReducerCallbackId<LoginByIdentityArgs> {
    LoginByIdentityArgs::on_reducer(move |__identity, __addr, __status, __args| {
        let LoginByIdentityArgs {} = __args;
        __callback(__identity, __addr, __status);
    })
}

#[allow(unused)]
pub fn once_on_login_by_identity(
    __callback: impl FnOnce(&Identity, Option<Address>, &Status) + Send + 'static,
) -> ReducerCallbackId<LoginByIdentityArgs> {
    LoginByIdentityArgs::once_on_reducer(move |__identity, __addr, __status, __args| {
        let LoginByIdentityArgs {} = __args;
        __callback(__identity, __addr, __status);
    })
}

#[allow(unused)]
pub fn remove_on_login_by_identity(id: ReducerCallbackId<LoginByIdentityArgs>) {
    LoginByIdentityArgs::remove_on_reducer(id);
}