// THIS FILE IS AUTOMATICALLY GENERATED BY SPACETIMEDB. EDITS TO THIS FILE
// WILL NOT BE SAVED. MODIFY TABLES IN YOUR MODULE SOURCE CODE INSTEAD.

#![allow(unused, clippy::all)]
use spacetimedb_sdk::__codegen::{self as __sdk, __lib, __sats, __ws};

#[derive(__lib::ser::Serialize, __lib::de::Deserialize, Clone, PartialEq, Debug)]
#[sats(crate = __lib)]
pub(super) struct IncubatorUpdateCoreArgs {}

impl From<IncubatorUpdateCoreArgs> for super::Reducer {
    fn from(args: IncubatorUpdateCoreArgs) -> Self {
        Self::IncubatorUpdateCore
    }
}

impl __sdk::InModule for IncubatorUpdateCoreArgs {
    type Module = super::RemoteModule;
}

pub struct IncubatorUpdateCoreCallbackId(__sdk::CallbackId);

#[allow(non_camel_case_types)]
/// Extension trait for access to the reducer `incubator_update_core`.
///
/// Implemented for [`super::RemoteReducers`].
pub trait incubator_update_core {
    /// Request that the remote module invoke the reducer `incubator_update_core` to run as soon as possible.
    ///
    /// This method returns immediately, and errors only if we are unable to send the request.
    /// The reducer will run asynchronously in the future,
    ///  and its status can be observed by listening for [`Self::on_incubator_update_core`] callbacks.
    fn incubator_update_core(&self) -> __sdk::Result<()>;
    /// Register a callback to run whenever we are notified of an invocation of the reducer `incubator_update_core`.
    ///
    /// Callbacks should inspect the [`__sdk::ReducerEvent`] contained in the [`super::ReducerEventContext`]
    /// to determine the reducer's status.
    ///
    /// The returned [`IncubatorUpdateCoreCallbackId`] can be passed to [`Self::remove_on_incubator_update_core`]
    /// to cancel the callback.
    fn on_incubator_update_core(
        &self,
        callback: impl FnMut(&super::ReducerEventContext) + Send + 'static,
    ) -> IncubatorUpdateCoreCallbackId;
    /// Cancel a callback previously registered by [`Self::on_incubator_update_core`],
    /// causing it not to run in the future.
    fn remove_on_incubator_update_core(&self, callback: IncubatorUpdateCoreCallbackId);
}

impl incubator_update_core for super::RemoteReducers {
    fn incubator_update_core(&self) -> __sdk::Result<()> {
        self.imp
            .call_reducer("incubator_update_core", IncubatorUpdateCoreArgs {})
    }
    fn on_incubator_update_core(
        &self,
        mut callback: impl FnMut(&super::ReducerEventContext) + Send + 'static,
    ) -> IncubatorUpdateCoreCallbackId {
        IncubatorUpdateCoreCallbackId(self.imp.on_reducer(
            "incubator_update_core",
            Box::new(move |ctx: &super::ReducerEventContext| {
                let super::ReducerEventContext {
                    event:
                        __sdk::ReducerEvent {
                            reducer: super::Reducer::IncubatorUpdateCore {},
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
    fn remove_on_incubator_update_core(&self, callback: IncubatorUpdateCoreCallbackId) {
        self.imp
            .remove_on_reducer("incubator_update_core", callback.0)
    }
}

#[allow(non_camel_case_types)]
#[doc(hidden)]
/// Extension trait for setting the call-flags for the reducer `incubator_update_core`.
///
/// Implemented for [`super::SetReducerFlags`].
///
/// This type is currently unstable and may be removed without a major version bump.
pub trait set_flags_for_incubator_update_core {
    /// Set the call-reducer flags for the reducer `incubator_update_core` to `flags`.
    ///
    /// This type is currently unstable and may be removed without a major version bump.
    fn incubator_update_core(&self, flags: __ws::CallReducerFlags);
}

impl set_flags_for_incubator_update_core for super::SetReducerFlags {
    fn incubator_update_core(&self, flags: __ws::CallReducerFlags) {
        self.imp
            .set_call_reducer_flags("incubator_update_core", flags);
    }
}
