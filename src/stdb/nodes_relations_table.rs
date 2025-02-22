// THIS FILE IS AUTOMATICALLY GENERATED BY SPACETIMEDB. EDITS TO THIS FILE
// WILL NOT BE SAVED. MODIFY TABLES IN RUST INSTEAD.

#![allow(unused)]
use super::t_node_relation_type::TNodeRelation;
use spacetimedb_sdk::__codegen::{
    self as __sdk, __lib, __sats, __ws,
    anyhow::{self as __anyhow, Context as _},
};

/// Table handle for the table `nodes_relations`.
///
/// Obtain a handle from the [`NodesRelationsTableAccess::nodes_relations`] method on [`super::RemoteTables`],
/// like `ctx.db.nodes_relations()`.
///
/// Users are encouraged not to explicitly reference this type,
/// but to directly chain method calls,
/// like `ctx.db.nodes_relations().on_insert(...)`.
pub struct NodesRelationsTableHandle<'ctx> {
    imp: __sdk::TableHandle<TNodeRelation>,
    ctx: std::marker::PhantomData<&'ctx super::RemoteTables>,
}

#[allow(non_camel_case_types)]
/// Extension trait for access to the table `nodes_relations`.
///
/// Implemented for [`super::RemoteTables`].
pub trait NodesRelationsTableAccess {
    #[allow(non_snake_case)]
    /// Obtain a [`NodesRelationsTableHandle`], which mediates access to the table `nodes_relations`.
    fn nodes_relations(&self) -> NodesRelationsTableHandle<'_>;
}

impl NodesRelationsTableAccess for super::RemoteTables {
    fn nodes_relations(&self) -> NodesRelationsTableHandle<'_> {
        NodesRelationsTableHandle {
            imp: self.imp.get_table::<TNodeRelation>("nodes_relations"),
            ctx: std::marker::PhantomData,
        }
    }
}

pub struct NodesRelationsInsertCallbackId(__sdk::CallbackId);
pub struct NodesRelationsDeleteCallbackId(__sdk::CallbackId);

impl<'ctx> __sdk::Table for NodesRelationsTableHandle<'ctx> {
    type Row = TNodeRelation;
    type EventContext = super::EventContext;

    fn count(&self) -> u64 {
        self.imp.count()
    }
    fn iter(&self) -> impl Iterator<Item = TNodeRelation> + '_ {
        self.imp.iter()
    }

    type InsertCallbackId = NodesRelationsInsertCallbackId;

    fn on_insert(
        &self,
        callback: impl FnMut(&Self::EventContext, &Self::Row) + Send + 'static,
    ) -> NodesRelationsInsertCallbackId {
        NodesRelationsInsertCallbackId(self.imp.on_insert(Box::new(callback)))
    }

    fn remove_on_insert(&self, callback: NodesRelationsInsertCallbackId) {
        self.imp.remove_on_insert(callback.0)
    }

    type DeleteCallbackId = NodesRelationsDeleteCallbackId;

    fn on_delete(
        &self,
        callback: impl FnMut(&Self::EventContext, &Self::Row) + Send + 'static,
    ) -> NodesRelationsDeleteCallbackId {
        NodesRelationsDeleteCallbackId(self.imp.on_delete(Box::new(callback)))
    }

    fn remove_on_delete(&self, callback: NodesRelationsDeleteCallbackId) {
        self.imp.remove_on_delete(callback.0)
    }
}

#[doc(hidden)]
pub(super) fn register_table(client_cache: &mut __sdk::ClientCache<super::RemoteModule>) {
    let _table = client_cache.get_or_make_table::<TNodeRelation>("nodes_relations");
    _table.add_unique_constraint::<u64>("id", |row| &row.id);
}
pub struct NodesRelationsUpdateCallbackId(__sdk::CallbackId);

impl<'ctx> __sdk::TableWithPrimaryKey for NodesRelationsTableHandle<'ctx> {
    type UpdateCallbackId = NodesRelationsUpdateCallbackId;

    fn on_update(
        &self,
        callback: impl FnMut(&Self::EventContext, &Self::Row, &Self::Row) + Send + 'static,
    ) -> NodesRelationsUpdateCallbackId {
        NodesRelationsUpdateCallbackId(self.imp.on_update(Box::new(callback)))
    }

    fn remove_on_update(&self, callback: NodesRelationsUpdateCallbackId) {
        self.imp.remove_on_update(callback.0)
    }
}

#[doc(hidden)]
pub(super) fn parse_table_update(
    raw_updates: __ws::TableUpdate<__ws::BsatnFormat>,
) -> __anyhow::Result<__sdk::TableUpdate<TNodeRelation>> {
    __sdk::TableUpdate::parse_table_update_with_primary_key::<u64>(
        raw_updates,
        |row: &TNodeRelation| &row.id,
    )
    .context("Failed to parse table update for table \"nodes_relations\"")
}

/// Access to the `id` unique index on the table `nodes_relations`,
/// which allows point queries on the field of the same name
/// via the [`NodesRelationsIdUnique::find`] method.
///
/// Users are encouraged not to explicitly reference this type,
/// but to directly chain method calls,
/// like `ctx.db.nodes_relations().id().find(...)`.
pub struct NodesRelationsIdUnique<'ctx> {
    imp: __sdk::UniqueConstraintHandle<TNodeRelation, u64>,
    phantom: std::marker::PhantomData<&'ctx super::RemoteTables>,
}

impl<'ctx> NodesRelationsTableHandle<'ctx> {
    /// Get a handle on the `id` unique index on the table `nodes_relations`.
    pub fn id(&self) -> NodesRelationsIdUnique<'ctx> {
        NodesRelationsIdUnique {
            imp: self.imp.get_unique_constraint::<u64>("id"),
            phantom: std::marker::PhantomData,
        }
    }
}

impl<'ctx> NodesRelationsIdUnique<'ctx> {
    /// Find the subscribed row whose `id` column value is equal to `col_val`,
    /// if such a row is present in the client cache.
    pub fn find(&self, col_val: &u64) -> Option<TNodeRelation> {
        self.imp.find(col_val)
    }
}
