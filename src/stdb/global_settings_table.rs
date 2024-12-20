// THIS FILE IS AUTOMATICALLY GENERATED BY SPACETIMEDB. EDITS TO THIS FILE
// WILL NOT BE SAVED. MODIFY TABLES IN RUST INSTEAD.

#![allow(unused)]
use super::global_settings_type::GlobalSettings;
use spacetimedb_sdk::{
    self as __sdk,
    anyhow::{self as __anyhow, Context as _},
    lib as __lib, sats as __sats, ws_messages as __ws,
};

/// Table handle for the table `global_settings`.
///
/// Obtain a handle from the [`GlobalSettingsTableAccess::global_settings`] method on [`super::RemoteTables`],
/// like `ctx.db.global_settings()`.
///
/// Users are encouraged not to explicitly reference this type,
/// but to directly chain method calls,
/// like `ctx.db.global_settings().on_insert(...)`.
pub struct GlobalSettingsTableHandle<'ctx> {
    imp: __sdk::db_connection::TableHandle<GlobalSettings>,
    ctx: std::marker::PhantomData<&'ctx super::RemoteTables>,
}

#[allow(non_camel_case_types)]
/// Extension trait for access to the table `global_settings`.
///
/// Implemented for [`super::RemoteTables`].
pub trait GlobalSettingsTableAccess {
    #[allow(non_snake_case)]
    /// Obtain a [`GlobalSettingsTableHandle`], which mediates access to the table `global_settings`.
    fn global_settings(&self) -> GlobalSettingsTableHandle<'_>;
}

impl GlobalSettingsTableAccess for super::RemoteTables {
    fn global_settings(&self) -> GlobalSettingsTableHandle<'_> {
        GlobalSettingsTableHandle {
            imp: self.imp.get_table::<GlobalSettings>("global_settings"),
            ctx: std::marker::PhantomData,
        }
    }
}

pub struct GlobalSettingsInsertCallbackId(__sdk::callbacks::CallbackId);
pub struct GlobalSettingsDeleteCallbackId(__sdk::callbacks::CallbackId);

impl<'ctx> __sdk::table::Table for GlobalSettingsTableHandle<'ctx> {
    type Row = GlobalSettings;
    type EventContext = super::EventContext;

    fn count(&self) -> u64 {
        self.imp.count()
    }
    fn iter(&self) -> impl Iterator<Item = GlobalSettings> + '_ {
        self.imp.iter()
    }

    type InsertCallbackId = GlobalSettingsInsertCallbackId;

    fn on_insert(
        &self,
        callback: impl FnMut(&Self::EventContext, &Self::Row) + Send + 'static,
    ) -> GlobalSettingsInsertCallbackId {
        GlobalSettingsInsertCallbackId(self.imp.on_insert(Box::new(callback)))
    }

    fn remove_on_insert(&self, callback: GlobalSettingsInsertCallbackId) {
        self.imp.remove_on_insert(callback.0)
    }

    type DeleteCallbackId = GlobalSettingsDeleteCallbackId;

    fn on_delete(
        &self,
        callback: impl FnMut(&Self::EventContext, &Self::Row) + Send + 'static,
    ) -> GlobalSettingsDeleteCallbackId {
        GlobalSettingsDeleteCallbackId(self.imp.on_delete(Box::new(callback)))
    }

    fn remove_on_delete(&self, callback: GlobalSettingsDeleteCallbackId) {
        self.imp.remove_on_delete(callback.0)
    }
}

#[doc(hidden)]
pub(super) fn parse_table_update(
    raw_updates: __ws::TableUpdate<__ws::BsatnFormat>,
) -> __anyhow::Result<__sdk::spacetime_module::TableUpdate<GlobalSettings>> {
    __sdk::spacetime_module::TableUpdate::parse_table_update_no_primary_key(raw_updates)
        .context("Failed to parse table update for table \"global_settings\"")
}

/// Access to the `always_zero` unique index on the table `global_settings`,
/// which allows point queries on the field of the same name
/// via the [`GlobalSettingsAlwaysZeroUnique::find`] method.
///
/// Users are encouraged not to explicitly reference this type,
/// but to directly chain method calls,
/// like `ctx.db.global_settings().always_zero().find(...)`.
pub struct GlobalSettingsAlwaysZeroUnique<'ctx> {
    imp: __sdk::client_cache::UniqueConstraint<GlobalSettings, u32>,
    phantom: std::marker::PhantomData<&'ctx super::RemoteTables>,
}

impl<'ctx> GlobalSettingsTableHandle<'ctx> {
    /// Get a handle on the `always_zero` unique index on the table `global_settings`.
    pub fn always_zero(&self) -> GlobalSettingsAlwaysZeroUnique<'ctx> {
        GlobalSettingsAlwaysZeroUnique {
            imp: self
                .imp
                .get_unique_constraint::<u32>("always_zero", |row| &row.always_zero),
            phantom: std::marker::PhantomData,
        }
    }
}

impl<'ctx> GlobalSettingsAlwaysZeroUnique<'ctx> {
    /// Find the subscribed row whose `always_zero` column value is equal to `col_val`,
    /// if such a row is present in the client cache.
    pub fn find(&self, col_val: &u32) -> Option<GlobalSettings> {
        self.imp.find(col_val)
    }
}
