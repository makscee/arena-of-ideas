// THIS FILE IS AUTOMATICALLY GENERATED BY SPACETIMEDB. EDITS TO THIS FILE
// WILL NOT BE SAVED. MODIFY TABLES IN YOUR MODULE SOURCE CODE INSTEAD.

#![allow(unused, clippy::all)]
use super::t_node_type::TNode;
use spacetimedb_sdk::__codegen::{self as __sdk, __lib, __sats, __ws};

/// Table handle for the table `nodes_world`.
///
/// Obtain a handle from the [`NodesWorldTableAccess::nodes_world`] method on [`super::RemoteTables`],
/// like `ctx.db.nodes_world()`.
///
/// Users are encouraged not to explicitly reference this type,
/// but to directly chain method calls,
/// like `ctx.db.nodes_world().on_insert(...)`.
pub struct NodesWorldTableHandle<'ctx> {
    imp: __sdk::TableHandle<TNode>,
    ctx: std::marker::PhantomData<&'ctx super::RemoteTables>,
}

#[allow(non_camel_case_types)]
/// Extension trait for access to the table `nodes_world`.
///
/// Implemented for [`super::RemoteTables`].
pub trait NodesWorldTableAccess {
    #[allow(non_snake_case)]
    /// Obtain a [`NodesWorldTableHandle`], which mediates access to the table `nodes_world`.
    fn nodes_world(&self) -> NodesWorldTableHandle<'_>;
}

impl NodesWorldTableAccess for super::RemoteTables {
    fn nodes_world(&self) -> NodesWorldTableHandle<'_> {
        NodesWorldTableHandle {
            imp: self.imp.get_table::<TNode>("nodes_world"),
            ctx: std::marker::PhantomData,
        }
    }
}

pub struct NodesWorldInsertCallbackId(__sdk::CallbackId);
pub struct NodesWorldDeleteCallbackId(__sdk::CallbackId);

impl<'ctx> __sdk::Table for NodesWorldTableHandle<'ctx> {
    type Row = TNode;
    type EventContext = super::EventContext;

    fn count(&self) -> u64 {
        self.imp.count()
    }
    fn iter(&self) -> impl Iterator<Item = TNode> + '_ {
        self.imp.iter()
    }

    type InsertCallbackId = NodesWorldInsertCallbackId;

    fn on_insert(
        &self,
        callback: impl FnMut(&Self::EventContext, &Self::Row) + Send + 'static,
    ) -> NodesWorldInsertCallbackId {
        NodesWorldInsertCallbackId(self.imp.on_insert(Box::new(callback)))
    }

    fn remove_on_insert(&self, callback: NodesWorldInsertCallbackId) {
        self.imp.remove_on_insert(callback.0)
    }

    type DeleteCallbackId = NodesWorldDeleteCallbackId;

    fn on_delete(
        &self,
        callback: impl FnMut(&Self::EventContext, &Self::Row) + Send + 'static,
    ) -> NodesWorldDeleteCallbackId {
        NodesWorldDeleteCallbackId(self.imp.on_delete(Box::new(callback)))
    }

    fn remove_on_delete(&self, callback: NodesWorldDeleteCallbackId) {
        self.imp.remove_on_delete(callback.0)
    }
}

#[doc(hidden)]
pub(super) fn register_table(client_cache: &mut __sdk::ClientCache<super::RemoteModule>) {
    let _table = client_cache.get_or_make_table::<TNode>("nodes_world");
    _table.add_unique_constraint::<u64>("id", |row| &row.id);
}
pub struct NodesWorldUpdateCallbackId(__sdk::CallbackId);

impl<'ctx> __sdk::TableWithPrimaryKey for NodesWorldTableHandle<'ctx> {
    type UpdateCallbackId = NodesWorldUpdateCallbackId;

    fn on_update(
        &self,
        callback: impl FnMut(&Self::EventContext, &Self::Row, &Self::Row) + Send + 'static,
    ) -> NodesWorldUpdateCallbackId {
        NodesWorldUpdateCallbackId(self.imp.on_update(Box::new(callback)))
    }

    fn remove_on_update(&self, callback: NodesWorldUpdateCallbackId) {
        self.imp.remove_on_update(callback.0)
    }
}

#[doc(hidden)]
pub(super) fn parse_table_update(
    raw_updates: __ws::TableUpdate<__ws::BsatnFormat>,
) -> __sdk::Result<__sdk::TableUpdate<TNode>> {
    __sdk::TableUpdate::parse_table_update(raw_updates).map_err(|e| {
        __sdk::InternalError::failed_parse("TableUpdate<TNode>", "TableUpdate")
            .with_cause(e)
            .into()
    })
}

/// Access to the `id` unique index on the table `nodes_world`,
/// which allows point queries on the field of the same name
/// via the [`NodesWorldIdUnique::find`] method.
///
/// Users are encouraged not to explicitly reference this type,
/// but to directly chain method calls,
/// like `ctx.db.nodes_world().id().find(...)`.
pub struct NodesWorldIdUnique<'ctx> {
    imp: __sdk::UniqueConstraintHandle<TNode, u64>,
    phantom: std::marker::PhantomData<&'ctx super::RemoteTables>,
}

impl<'ctx> NodesWorldTableHandle<'ctx> {
    /// Get a handle on the `id` unique index on the table `nodes_world`.
    pub fn id(&self) -> NodesWorldIdUnique<'ctx> {
        NodesWorldIdUnique {
            imp: self.imp.get_unique_constraint::<u64>("id"),
            phantom: std::marker::PhantomData,
        }
    }
}

impl<'ctx> NodesWorldIdUnique<'ctx> {
    /// Find the subscribed row whose `id` column value is equal to `col_val`,
    /// if such a row is present in the client cache.
    pub fn find(&self, col_val: &u64) -> Option<TNode> {
        self.imp.find(col_val)
    }
}
