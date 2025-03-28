// THIS FILE IS AUTOMATICALLY GENERATED BY SPACETIMEDB. EDITS TO THIS FILE
// WILL NOT BE SAVED. MODIFY TABLES IN YOUR MODULE SOURCE CODE INSTEAD.

#![allow(unused, clippy::all)]
use spacetimedb_sdk::__codegen::{self as __sdk, __lib, __sats, __ws};

#[derive(__lib::ser::Serialize, __lib::de::Deserialize, Clone, PartialEq, Debug)]
#[sats(crate = __lib)]
pub(super) struct MatchReorderArgs {
    pub slot: u8,
    pub target: u8,
}

impl From<MatchReorderArgs> for super::Reducer {
    fn from(args: MatchReorderArgs) -> Self {
        Self::MatchReorder {
            slot: args.slot,
            target: args.target,
        }
    }
}

impl __sdk::InModule for MatchReorderArgs {
    type Module = super::RemoteModule;
}

pub struct MatchReorderCallbackId(__sdk::CallbackId);

#[allow(non_camel_case_types)]
/// Extension trait for access to the reducer `match_reorder`.
///
/// Implemented for [`super::RemoteReducers`].
pub trait match_reorder {
    /// Request that the remote module invoke the reducer `match_reorder` to run as soon as possible.
    ///
    /// This method returns immediately, and errors only if we are unable to send the request.
    /// The reducer will run asynchronously in the future,
    ///  and its status can be observed by listening for [`Self::on_match_reorder`] callbacks.
    fn match_reorder(&self, slot: u8, target: u8) -> __sdk::Result<()>;
    /// Register a callback to run whenever we are notified of an invocation of the reducer `match_reorder`.
    ///
    /// Callbacks should inspect the [`__sdk::ReducerEvent`] contained in the [`super::ReducerEventContext`]
    /// to determine the reducer's status.
    ///
    /// The returned [`MatchReorderCallbackId`] can be passed to [`Self::remove_on_match_reorder`]
    /// to cancel the callback.
    fn on_match_reorder(
        &self,
        callback: impl FnMut(&super::ReducerEventContext, &u8, &u8) + Send + 'static,
    ) -> MatchReorderCallbackId;
    /// Cancel a callback previously registered by [`Self::on_match_reorder`],
    /// causing it not to run in the future.
    fn remove_on_match_reorder(&self, callback: MatchReorderCallbackId);
}

impl match_reorder for super::RemoteReducers {
    fn match_reorder(&self, slot: u8, target: u8) -> __sdk::Result<()> {
        self.imp
            .call_reducer("match_reorder", MatchReorderArgs { slot, target })
    }
    fn on_match_reorder(
        &self,
        mut callback: impl FnMut(&super::ReducerEventContext, &u8, &u8) + Send + 'static,
    ) -> MatchReorderCallbackId {
        MatchReorderCallbackId(self.imp.on_reducer(
            "match_reorder",
            Box::new(move |ctx: &super::ReducerEventContext| {
                let super::ReducerEventContext {
                    event:
                        __sdk::ReducerEvent {
                            reducer: super::Reducer::MatchReorder { slot, target },
                            ..
                        },
                    ..
                } = ctx
                else {
                    unreachable!()
                };
                callback(ctx, slot, target)
            }),
        ))
    }
    fn remove_on_match_reorder(&self, callback: MatchReorderCallbackId) {
        self.imp.remove_on_reducer("match_reorder", callback.0)
    }
}

#[allow(non_camel_case_types)]
#[doc(hidden)]
/// Extension trait for setting the call-flags for the reducer `match_reorder`.
///
/// Implemented for [`super::SetReducerFlags`].
///
/// This type is currently unstable and may be removed without a major version bump.
pub trait set_flags_for_match_reorder {
    /// Set the call-reducer flags for the reducer `match_reorder` to `flags`.
    ///
    /// This type is currently unstable and may be removed without a major version bump.
    fn match_reorder(&self, flags: __ws::CallReducerFlags);
}

impl set_flags_for_match_reorder for super::SetReducerFlags {
    fn match_reorder(&self, flags: __ws::CallReducerFlags) {
        self.imp.set_call_reducer_flags("match_reorder", flags);
    }
}
