// THIS FILE IS AUTOMATICALLY GENERATED BY SPACETIMEDB. EDITS TO THIS FILE
// WILL NOT BE SAVED. MODIFY TABLES IN YOUR MODULE SOURCE CODE INSTEAD.

#![allow(unused, clippy::all)]
use spacetimedb_sdk::__codegen::{self as __sdk, __lib, __sats, __ws};

#[derive(__lib::ser::Serialize, __lib::de::Deserialize, Clone, PartialEq, Debug)]
#[sats(crate = __lib)]
pub(super) struct MatchSubmitBattleResultArgs {
    pub id: u64,
    pub result: bool,
    pub hash: u64,
}

impl From<MatchSubmitBattleResultArgs> for super::Reducer {
    fn from(args: MatchSubmitBattleResultArgs) -> Self {
        Self::MatchSubmitBattleResult {
            id: args.id,
            result: args.result,
            hash: args.hash,
        }
    }
}

impl __sdk::InModule for MatchSubmitBattleResultArgs {
    type Module = super::RemoteModule;
}

pub struct MatchSubmitBattleResultCallbackId(__sdk::CallbackId);

#[allow(non_camel_case_types)]
/// Extension trait for access to the reducer `match_submit_battle_result`.
///
/// Implemented for [`super::RemoteReducers`].
pub trait match_submit_battle_result {
    /// Request that the remote module invoke the reducer `match_submit_battle_result` to run as soon as possible.
    ///
    /// This method returns immediately, and errors only if we are unable to send the request.
    /// The reducer will run asynchronously in the future,
    ///  and its status can be observed by listening for [`Self::on_match_submit_battle_result`] callbacks.
    fn match_submit_battle_result(&self, id: u64, result: bool, hash: u64) -> __sdk::Result<()>;
    /// Register a callback to run whenever we are notified of an invocation of the reducer `match_submit_battle_result`.
    ///
    /// Callbacks should inspect the [`__sdk::ReducerEvent`] contained in the [`super::ReducerEventContext`]
    /// to determine the reducer's status.
    ///
    /// The returned [`MatchSubmitBattleResultCallbackId`] can be passed to [`Self::remove_on_match_submit_battle_result`]
    /// to cancel the callback.
    fn on_match_submit_battle_result(
        &self,
        callback: impl FnMut(&super::ReducerEventContext, &u64, &bool, &u64) + Send + 'static,
    ) -> MatchSubmitBattleResultCallbackId;
    /// Cancel a callback previously registered by [`Self::on_match_submit_battle_result`],
    /// causing it not to run in the future.
    fn remove_on_match_submit_battle_result(&self, callback: MatchSubmitBattleResultCallbackId);
}

impl match_submit_battle_result for super::RemoteReducers {
    fn match_submit_battle_result(&self, id: u64, result: bool, hash: u64) -> __sdk::Result<()> {
        self.imp.call_reducer(
            "match_submit_battle_result",
            MatchSubmitBattleResultArgs { id, result, hash },
        )
    }
    fn on_match_submit_battle_result(
        &self,
        mut callback: impl FnMut(&super::ReducerEventContext, &u64, &bool, &u64) + Send + 'static,
    ) -> MatchSubmitBattleResultCallbackId {
        MatchSubmitBattleResultCallbackId(self.imp.on_reducer(
            "match_submit_battle_result",
            Box::new(move |ctx: &super::ReducerEventContext| {
                let super::ReducerEventContext {
                    event:
                        __sdk::ReducerEvent {
                            reducer: super::Reducer::MatchSubmitBattleResult { id, result, hash },
                            ..
                        },
                    ..
                } = ctx
                else {
                    unreachable!()
                };
                callback(ctx, id, result, hash)
            }),
        ))
    }
    fn remove_on_match_submit_battle_result(&self, callback: MatchSubmitBattleResultCallbackId) {
        self.imp
            .remove_on_reducer("match_submit_battle_result", callback.0)
    }
}

#[allow(non_camel_case_types)]
#[doc(hidden)]
/// Extension trait for setting the call-flags for the reducer `match_submit_battle_result`.
///
/// Implemented for [`super::SetReducerFlags`].
///
/// This type is currently unstable and may be removed without a major version bump.
pub trait set_flags_for_match_submit_battle_result {
    /// Set the call-reducer flags for the reducer `match_submit_battle_result` to `flags`.
    ///
    /// This type is currently unstable and may be removed without a major version bump.
    fn match_submit_battle_result(&self, flags: __ws::CallReducerFlags);
}

impl set_flags_for_match_submit_battle_result for super::SetReducerFlags {
    fn match_submit_battle_result(&self, flags: __ws::CallReducerFlags) {
        self.imp
            .set_call_reducer_flags("match_submit_battle_result", flags);
    }
}
