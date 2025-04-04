// THIS FILE IS AUTOMATICALLY GENERATED BY SPACETIMEDB. EDITS TO THIS FILE
// WILL NOT BE SAVED. MODIFY TABLES IN YOUR MODULE SOURCE CODE INSTEAD.

#![allow(unused, clippy::all)]
use super::t_battle_type::TBattle;
use spacetimedb_sdk::__codegen::{self as __sdk, __lib, __sats, __ws};

/// Table handle for the table `battle`.
///
/// Obtain a handle from the [`BattleTableAccess::battle`] method on [`super::RemoteTables`],
/// like `ctx.db.battle()`.
///
/// Users are encouraged not to explicitly reference this type,
/// but to directly chain method calls,
/// like `ctx.db.battle().on_insert(...)`.
pub struct BattleTableHandle<'ctx> {
    imp: __sdk::TableHandle<TBattle>,
    ctx: std::marker::PhantomData<&'ctx super::RemoteTables>,
}

#[allow(non_camel_case_types)]
/// Extension trait for access to the table `battle`.
///
/// Implemented for [`super::RemoteTables`].
pub trait BattleTableAccess {
    #[allow(non_snake_case)]
    /// Obtain a [`BattleTableHandle`], which mediates access to the table `battle`.
    fn battle(&self) -> BattleTableHandle<'_>;
}

impl BattleTableAccess for super::RemoteTables {
    fn battle(&self) -> BattleTableHandle<'_> {
        BattleTableHandle {
            imp: self.imp.get_table::<TBattle>("battle"),
            ctx: std::marker::PhantomData,
        }
    }
}

pub struct BattleInsertCallbackId(__sdk::CallbackId);
pub struct BattleDeleteCallbackId(__sdk::CallbackId);

impl<'ctx> __sdk::Table for BattleTableHandle<'ctx> {
    type Row = TBattle;
    type EventContext = super::EventContext;

    fn count(&self) -> u64 {
        self.imp.count()
    }
    fn iter(&self) -> impl Iterator<Item = TBattle> + '_ {
        self.imp.iter()
    }

    type InsertCallbackId = BattleInsertCallbackId;

    fn on_insert(
        &self,
        callback: impl FnMut(&Self::EventContext, &Self::Row) + Send + 'static,
    ) -> BattleInsertCallbackId {
        BattleInsertCallbackId(self.imp.on_insert(Box::new(callback)))
    }

    fn remove_on_insert(&self, callback: BattleInsertCallbackId) {
        self.imp.remove_on_insert(callback.0)
    }

    type DeleteCallbackId = BattleDeleteCallbackId;

    fn on_delete(
        &self,
        callback: impl FnMut(&Self::EventContext, &Self::Row) + Send + 'static,
    ) -> BattleDeleteCallbackId {
        BattleDeleteCallbackId(self.imp.on_delete(Box::new(callback)))
    }

    fn remove_on_delete(&self, callback: BattleDeleteCallbackId) {
        self.imp.remove_on_delete(callback.0)
    }
}

#[doc(hidden)]
pub(super) fn register_table(client_cache: &mut __sdk::ClientCache<super::RemoteModule>) {
    let _table = client_cache.get_or_make_table::<TBattle>("battle");
    _table.add_unique_constraint::<u64>("id", |row| &row.id);
}
pub struct BattleUpdateCallbackId(__sdk::CallbackId);

impl<'ctx> __sdk::TableWithPrimaryKey for BattleTableHandle<'ctx> {
    type UpdateCallbackId = BattleUpdateCallbackId;

    fn on_update(
        &self,
        callback: impl FnMut(&Self::EventContext, &Self::Row, &Self::Row) + Send + 'static,
    ) -> BattleUpdateCallbackId {
        BattleUpdateCallbackId(self.imp.on_update(Box::new(callback)))
    }

    fn remove_on_update(&self, callback: BattleUpdateCallbackId) {
        self.imp.remove_on_update(callback.0)
    }
}

#[doc(hidden)]
pub(super) fn parse_table_update(
    raw_updates: __ws::TableUpdate<__ws::BsatnFormat>,
) -> __sdk::Result<__sdk::TableUpdate<TBattle>> {
    __sdk::TableUpdate::parse_table_update(raw_updates).map_err(|e| {
        __sdk::InternalError::failed_parse("TableUpdate<TBattle>", "TableUpdate")
            .with_cause(e)
            .into()
    })
}

/// Access to the `id` unique index on the table `battle`,
/// which allows point queries on the field of the same name
/// via the [`BattleIdUnique::find`] method.
///
/// Users are encouraged not to explicitly reference this type,
/// but to directly chain method calls,
/// like `ctx.db.battle().id().find(...)`.
pub struct BattleIdUnique<'ctx> {
    imp: __sdk::UniqueConstraintHandle<TBattle, u64>,
    phantom: std::marker::PhantomData<&'ctx super::RemoteTables>,
}

impl<'ctx> BattleTableHandle<'ctx> {
    /// Get a handle on the `id` unique index on the table `battle`.
    pub fn id(&self) -> BattleIdUnique<'ctx> {
        BattleIdUnique {
            imp: self.imp.get_unique_constraint::<u64>("id"),
            phantom: std::marker::PhantomData,
        }
    }
}

impl<'ctx> BattleIdUnique<'ctx> {
    /// Find the subscribed row whose `id` column value is equal to `col_val`,
    /// if such a row is present in the client cache.
    pub fn find(&self, col_val: &u64) -> Option<TBattle> {
        self.imp.find(col_val)
    }
}
