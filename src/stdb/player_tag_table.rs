// THIS FILE IS AUTOMATICALLY GENERATED BY SPACETIMEDB. EDITS TO THIS FILE
// WILL NOT BE SAVED. MODIFY TABLES IN RUST INSTEAD.

#![allow(unused)]
use super::t_player_tag_type::TPlayerTag;
use spacetimedb_sdk::{
    self as __sdk,
    anyhow::{self as __anyhow, Context as _},
    lib as __lib, sats as __sats, ws_messages as __ws,
};

/// Table handle for the table `player_tag`.
///
/// Obtain a handle from the [`PlayerTagTableAccess::player_tag`] method on [`super::RemoteTables`],
/// like `ctx.db.player_tag()`.
///
/// Users are encouraged not to explicitly reference this type,
/// but to directly chain method calls,
/// like `ctx.db.player_tag().on_insert(...)`.
pub struct PlayerTagTableHandle<'ctx> {
    imp: __sdk::db_connection::TableHandle<TPlayerTag>,
    ctx: std::marker::PhantomData<&'ctx super::RemoteTables>,
}

#[allow(non_camel_case_types)]
/// Extension trait for access to the table `player_tag`.
///
/// Implemented for [`super::RemoteTables`].
pub trait PlayerTagTableAccess {
    #[allow(non_snake_case)]
    /// Obtain a [`PlayerTagTableHandle`], which mediates access to the table `player_tag`.
    fn player_tag(&self) -> PlayerTagTableHandle<'_>;
}

impl PlayerTagTableAccess for super::RemoteTables {
    fn player_tag(&self) -> PlayerTagTableHandle<'_> {
        PlayerTagTableHandle {
            imp: self.imp.get_table::<TPlayerTag>("player_tag"),
            ctx: std::marker::PhantomData,
        }
    }
}

pub struct PlayerTagInsertCallbackId(__sdk::callbacks::CallbackId);
pub struct PlayerTagDeleteCallbackId(__sdk::callbacks::CallbackId);

impl<'ctx> __sdk::table::Table for PlayerTagTableHandle<'ctx> {
    type Row = TPlayerTag;
    type EventContext = super::EventContext;

    fn count(&self) -> u64 {
        self.imp.count()
    }
    fn iter(&self) -> impl Iterator<Item = TPlayerTag> + '_ {
        self.imp.iter()
    }

    type InsertCallbackId = PlayerTagInsertCallbackId;

    fn on_insert(
        &self,
        callback: impl FnMut(&Self::EventContext, &Self::Row) + Send + 'static,
    ) -> PlayerTagInsertCallbackId {
        PlayerTagInsertCallbackId(self.imp.on_insert(Box::new(callback)))
    }

    fn remove_on_insert(&self, callback: PlayerTagInsertCallbackId) {
        self.imp.remove_on_insert(callback.0)
    }

    type DeleteCallbackId = PlayerTagDeleteCallbackId;

    fn on_delete(
        &self,
        callback: impl FnMut(&Self::EventContext, &Self::Row) + Send + 'static,
    ) -> PlayerTagDeleteCallbackId {
        PlayerTagDeleteCallbackId(self.imp.on_delete(Box::new(callback)))
    }

    fn remove_on_delete(&self, callback: PlayerTagDeleteCallbackId) {
        self.imp.remove_on_delete(callback.0)
    }
}

pub struct PlayerTagUpdateCallbackId(__sdk::callbacks::CallbackId);

impl<'ctx> __sdk::table::TableWithPrimaryKey for PlayerTagTableHandle<'ctx> {
    type UpdateCallbackId = PlayerTagUpdateCallbackId;

    fn on_update(
        &self,
        callback: impl FnMut(&Self::EventContext, &Self::Row, &Self::Row) + Send + 'static,
    ) -> PlayerTagUpdateCallbackId {
        PlayerTagUpdateCallbackId(self.imp.on_update(Box::new(callback)))
    }

    fn remove_on_update(&self, callback: PlayerTagUpdateCallbackId) {
        self.imp.remove_on_update(callback.0)
    }
}

#[doc(hidden)]
pub(super) fn parse_table_update(
    raw_updates: __ws::TableUpdate<__ws::BsatnFormat>,
) -> __anyhow::Result<__sdk::spacetime_module::TableUpdate<TPlayerTag>> {
    __sdk::spacetime_module::TableUpdate::parse_table_update_with_primary_key::<u64>(
        raw_updates,
        |row: &TPlayerTag| &row.id,
    )
    .context("Failed to parse table update for table \"player_tag\"")
}

/// Access to the `id` unique index on the table `player_tag`,
/// which allows point queries on the field of the same name
/// via the [`PlayerTagIdUnique::find`] method.
///
/// Users are encouraged not to explicitly reference this type,
/// but to directly chain method calls,
/// like `ctx.db.player_tag().id().find(...)`.
pub struct PlayerTagIdUnique<'ctx> {
    imp: __sdk::client_cache::UniqueConstraint<TPlayerTag, u64>,
    phantom: std::marker::PhantomData<&'ctx super::RemoteTables>,
}

impl<'ctx> PlayerTagTableHandle<'ctx> {
    /// Get a handle on the `id` unique index on the table `player_tag`.
    pub fn id(&self) -> PlayerTagIdUnique<'ctx> {
        PlayerTagIdUnique {
            imp: self.imp.get_unique_constraint::<u64>("id", |row| &row.id),
            phantom: std::marker::PhantomData,
        }
    }
}

impl<'ctx> PlayerTagIdUnique<'ctx> {
    /// Find the subscribed row whose `id` column value is equal to `col_val`,
    /// if such a row is present in the client cache.
    pub fn find(&self, col_val: &u64) -> Option<TPlayerTag> {
        self.imp.find(col_val)
    }
}
