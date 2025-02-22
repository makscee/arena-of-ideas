// THIS FILE IS AUTOMATICALLY GENERATED BY SPACETIMEDB. EDITS TO THIS FILE
// WILL NOT BE SAVED. MODIFY TABLES IN RUST INSTEAD.

#![allow(unused)]
use spacetimedb_sdk::__codegen::{
    self as __sdk, __lib, __sats, __ws,
    anyhow::{self as __anyhow, Context as _},
};

#[derive(__lib::ser::Serialize, __lib::de::Deserialize, Clone, PartialEq, Debug)]
#[sats(crate = __lib)]
pub(super) struct SetNameArgs {
    pub name: String,
}

impl From<SetNameArgs> for super::Reducer {
    fn from(args: SetNameArgs) -> Self {
        Self::SetName { name: args.name }
    }
}

impl __sdk::InModule for SetNameArgs {
    type Module = super::RemoteModule;
}

pub struct SetNameCallbackId(__sdk::CallbackId);

#[allow(non_camel_case_types)]
/// Extension trait for access to the reducer `set_name`.
///
/// Implemented for [`super::RemoteReducers`].
pub trait set_name {
    /// Request that the remote module invoke the reducer `set_name` to run as soon as possible.
    ///
    /// This method returns immediately, and errors only if we are unable to send the request.
    /// The reducer will run asynchronously in the future,
    ///  and its status can be observed by listening for [`Self::on_set_name`] callbacks.
    fn set_name(&self, name: String) -> __anyhow::Result<()>;
    /// Register a callback to run whenever we are notified of an invocation of the reducer `set_name`.
    ///
    /// The [`super::EventContext`] passed to the `callback`
    /// will always have [`__sdk::Event::Reducer`] as its `event`,
    /// but it may or may not have terminated successfully and been committed.
    /// Callbacks should inspect the [`__sdk::ReducerEvent`] contained in the [`super::EventContext`]
    /// to determine the reducer's status.
    ///
    /// The returned [`SetNameCallbackId`] can be passed to [`Self::remove_on_set_name`]
    /// to cancel the callback.
    fn on_set_name(
        &self,
        callback: impl FnMut(&super::EventContext, &String) + Send + 'static,
    ) -> SetNameCallbackId;
    /// Cancel a callback previously registered by [`Self::on_set_name`],
    /// causing it not to run in the future.
    fn remove_on_set_name(&self, callback: SetNameCallbackId);
}

impl set_name for super::RemoteReducers {
    fn set_name(&self, name: String) -> __anyhow::Result<()> {
        self.imp.call_reducer("set_name", SetNameArgs { name })
    }
    fn on_set_name(
        &self,
        mut callback: impl FnMut(&super::EventContext, &String) + Send + 'static,
    ) -> SetNameCallbackId {
        SetNameCallbackId(self.imp.on_reducer(
            "set_name",
            Box::new(move |ctx: &super::EventContext| {
                let super::EventContext {
                    event:
                        __sdk::Event::Reducer(__sdk::ReducerEvent {
                            reducer: super::Reducer::SetName { name },
                            ..
                        }),
                    ..
                } = ctx
                else {
                    unreachable!()
                };
                callback(ctx, name)
            }),
        ))
    }
    fn remove_on_set_name(&self, callback: SetNameCallbackId) {
        self.imp.remove_on_reducer("set_name", callback.0)
    }
}

#[allow(non_camel_case_types)]
#[doc(hidden)]
/// Extension trait for setting the call-flags for the reducer `set_name`.
///
/// Implemented for [`super::SetReducerFlags`].
///
/// This type is currently unstable and may be removed without a major version bump.
pub trait set_flags_for_set_name {
    /// Set the call-reducer flags for the reducer `set_name` to `flags`.
    ///
    /// This type is currently unstable and may be removed without a major version bump.
    fn set_name(&self, flags: __ws::CallReducerFlags);
}

impl set_flags_for_set_name for super::SetReducerFlags {
    fn set_name(&self, flags: __ws::CallReducerFlags) {
        self.imp.set_call_reducer_flags("set_name", flags);
    }
}
