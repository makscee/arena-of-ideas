// THIS FILE IS AUTOMATICALLY GENERATED BY SPACETIMEDB. EDITS TO THIS FILE
// WILL NOT BE SAVED. MODIFY TABLES IN YOUR MODULE SOURCE CODE INSTEAD.

#![allow(unused, clippy::all)]
use spacetimedb_sdk::__codegen::{self as __sdk, __lib, __sats, __ws};

#[derive(__lib::ser::Serialize, __lib::de::Deserialize, Clone, PartialEq, Debug)]
#[sats(crate = __lib)]
pub(super) struct MatchInsertArgs {}

impl From<MatchInsertArgs> for super::Reducer {
    fn from(args: MatchInsertArgs) -> Self {
        Self::MatchInsert
    }
}

impl __sdk::InModule for MatchInsertArgs {
    type Module = super::RemoteModule;
}

pub struct MatchInsertCallbackId(__sdk::CallbackId);

#[allow(non_camel_case_types)]
/// Extension trait for access to the reducer `match_insert`.
///
/// Implemented for [`super::RemoteReducers`].
pub trait match_insert {
    /// Request that the remote module invoke the reducer `match_insert` to run as soon as possible.
    ///
    /// This method returns immediately, and errors only if we are unable to send the request.
    /// The reducer will run asynchronously in the future,
    ///  and its status can be observed by listening for [`Self::on_match_insert`] callbacks.
    fn match_insert(&self) -> __sdk::Result<()>;
    /// Register a callback to run whenever we are notified of an invocation of the reducer `match_insert`.
    ///
    /// Callbacks should inspect the [`__sdk::ReducerEvent`] contained in the [`super::ReducerEventContext`]
    /// to determine the reducer's status.
    ///
    /// The returned [`MatchInsertCallbackId`] can be passed to [`Self::remove_on_match_insert`]
    /// to cancel the callback.
    fn on_match_insert(
        &self,
        callback: impl FnMut(&super::ReducerEventContext) + Send + 'static,
    ) -> MatchInsertCallbackId;
    /// Cancel a callback previously registered by [`Self::on_match_insert`],
    /// causing it not to run in the future.
    fn remove_on_match_insert(&self, callback: MatchInsertCallbackId);
}

impl match_insert for super::RemoteReducers {
    fn match_insert(&self) -> __sdk::Result<()> {
        self.imp.call_reducer("match_insert", MatchInsertArgs {})
    }
    fn on_match_insert(
        &self,
        mut callback: impl FnMut(&super::ReducerEventContext) + Send + 'static,
    ) -> MatchInsertCallbackId {
        MatchInsertCallbackId(self.imp.on_reducer(
            "match_insert",
            Box::new(move |ctx: &super::ReducerEventContext| {
                let super::ReducerEventContext {
                    event:
                        __sdk::ReducerEvent {
                            reducer: super::Reducer::MatchInsert {},
                            ..
                        },
                    ..
                } = ctx
                else {
                    unreachable!()
                };
                callback(ctx)
            }),
        ))
    }
    fn remove_on_match_insert(&self, callback: MatchInsertCallbackId) {
        self.imp.remove_on_reducer("match_insert", callback.0)
    }
}

#[allow(non_camel_case_types)]
#[doc(hidden)]
/// Extension trait for setting the call-flags for the reducer `match_insert`.
///
/// Implemented for [`super::SetReducerFlags`].
///
/// This type is currently unstable and may be removed without a major version bump.
pub trait set_flags_for_match_insert {
    /// Set the call-reducer flags for the reducer `match_insert` to `flags`.
    ///
    /// This type is currently unstable and may be removed without a major version bump.
    fn match_insert(&self, flags: __ws::CallReducerFlags);
}

impl set_flags_for_match_insert for super::SetReducerFlags {
    fn match_insert(&self, flags: __ws::CallReducerFlags) {
        self.imp.set_call_reducer_flags("match_insert", flags);
    }
}
