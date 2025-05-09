// THIS FILE IS AUTOMATICALLY GENERATED BY SPACETIMEDB. EDITS TO THIS FILE
// WILL NOT BE SAVED. MODIFY TABLES IN YOUR MODULE SOURCE CODE INSTEAD.

#![allow(unused, clippy::all)]
use super::t_node_link_type::TNodeLink;
use spacetimedb_sdk::__codegen::{self as __sdk, __lib, __sats, __ws};

/// Table handle for the table `node_links`.
///
/// Obtain a handle from the [`NodeLinksTableAccess::node_links`] method on [`super::RemoteTables`],
/// like `ctx.db.node_links()`.
///
/// Users are encouraged not to explicitly reference this type,
/// but to directly chain method calls,
/// like `ctx.db.node_links().on_insert(...)`.
pub struct NodeLinksTableHandle<'ctx> {
    imp: __sdk::TableHandle<TNodeLink>,
    ctx: std::marker::PhantomData<&'ctx super::RemoteTables>,
}

#[allow(non_camel_case_types)]
/// Extension trait for access to the table `node_links`.
///
/// Implemented for [`super::RemoteTables`].
pub trait NodeLinksTableAccess {
    #[allow(non_snake_case)]
    /// Obtain a [`NodeLinksTableHandle`], which mediates access to the table `node_links`.
    fn node_links(&self) -> NodeLinksTableHandle<'_>;
}

impl NodeLinksTableAccess for super::RemoteTables {
    fn node_links(&self) -> NodeLinksTableHandle<'_> {
        NodeLinksTableHandle {
            imp: self.imp.get_table::<TNodeLink>("node_links"),
            ctx: std::marker::PhantomData,
        }
    }
}

pub struct NodeLinksInsertCallbackId(__sdk::CallbackId);
pub struct NodeLinksDeleteCallbackId(__sdk::CallbackId);

impl<'ctx> __sdk::Table for NodeLinksTableHandle<'ctx> {
    type Row = TNodeLink;
    type EventContext = super::EventContext;

    fn count(&self) -> u64 {
        self.imp.count()
    }
    fn iter(&self) -> impl Iterator<Item = TNodeLink> + '_ {
        self.imp.iter()
    }

    type InsertCallbackId = NodeLinksInsertCallbackId;

    fn on_insert(
        &self,
        callback: impl FnMut(&Self::EventContext, &Self::Row) + Send + 'static,
    ) -> NodeLinksInsertCallbackId {
        NodeLinksInsertCallbackId(self.imp.on_insert(Box::new(callback)))
    }

    fn remove_on_insert(&self, callback: NodeLinksInsertCallbackId) {
        self.imp.remove_on_insert(callback.0)
    }

    type DeleteCallbackId = NodeLinksDeleteCallbackId;

    fn on_delete(
        &self,
        callback: impl FnMut(&Self::EventContext, &Self::Row) + Send + 'static,
    ) -> NodeLinksDeleteCallbackId {
        NodeLinksDeleteCallbackId(self.imp.on_delete(Box::new(callback)))
    }

    fn remove_on_delete(&self, callback: NodeLinksDeleteCallbackId) {
        self.imp.remove_on_delete(callback.0)
    }
}

#[doc(hidden)]
pub(super) fn register_table(client_cache: &mut __sdk::ClientCache<super::RemoteModule>) {
    let _table = client_cache.get_or_make_table::<TNodeLink>("node_links");
}

#[doc(hidden)]
pub(super) fn parse_table_update(
    raw_updates: __ws::TableUpdate<__ws::BsatnFormat>,
) -> __sdk::Result<__sdk::TableUpdate<TNodeLink>> {
    __sdk::TableUpdate::parse_table_update(raw_updates).map_err(|e| {
        __sdk::InternalError::failed_parse("TableUpdate<TNodeLink>", "TableUpdate")
            .with_cause(e)
            .into()
    })
}
