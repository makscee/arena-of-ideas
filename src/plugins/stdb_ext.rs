use spacetimedb_sdk::{Event, Table};

use super::*;

pub trait TableSingletonExt: Table {
    fn current(&self) -> Self::Row {
        *Self::get_current(self).unwrap()
    }
    fn get_current(&self) -> Option<Box<Self::Row>> {
        Self::iter(self).exactly_one().ok().map(|d| Box::new(d))
    }
}

impl TableSingletonExt for GlobalDataTableHandle<'static> {}
impl TableSingletonExt for GlobalSettingsTableHandle<'static> {}

pub trait StdbStatusExt {
    fn on_success(&self, f: impl FnOnce() + Send + Sync + 'static);
    fn on_success_op(&self, f: impl FnOnce(&mut World) + Send + Sync + 'static);
    fn on_success_error(
        &self,
        f: impl FnOnce() + Send + Sync + 'static,
        f: impl FnOnce() + Send + Sync + 'static,
    );
    fn on_success_error_op(
        &self,
        f: impl FnOnce(&mut World) + Send + Sync + 'static,
        f: impl FnOnce(&mut World) + Send + Sync + 'static,
    );
    fn notify_error(&self);
}

impl<R> StdbStatusExt for Event<R> {
    fn on_success(&self, f: impl FnOnce() + Send + Sync + 'static) {
        match self {
            Event::Reducer(r) => match &r.status {
                spacetimedb_sdk::Status::Committed => f(),
                spacetimedb_sdk::Status::Failed(e) => e.notify_error_op(),
                _ => panic!(),
            },
            Event::SubscribeApplied | Event::UnsubscribeApplied => f(),
            Event::SubscribeError(e) => e.to_string().notify_error_op(),
            _ => panic!(),
        }
    }
    fn on_success_op(&self, f: impl FnOnce(&mut World) + Send + Sync + 'static) {
        self.on_success(move || OperationsPlugin::add(|w| f(w)));
    }
    fn on_success_error(
        &self,
        sfn: impl FnOnce() + Send + Sync + 'static,
        efn: impl FnOnce() + Send + Sync + 'static,
    ) {
        match self {
            Event::Reducer(r) => match &r.status {
                spacetimedb_sdk::Status::Committed => sfn(),
                spacetimedb_sdk::Status::Failed(e) => {
                    e.notify_error_op();
                    efn()
                }
                _ => panic!(),
            },
            Event::SubscribeApplied | Event::UnsubscribeApplied => sfn(),
            Event::SubscribeError(e) => {
                e.to_string().notify_error_op();
                efn()
            }
            _ => panic!(),
        }
    }
    fn on_success_error_op(
        &self,
        s: impl FnOnce(&mut World) + Send + Sync + 'static,
        e: impl FnOnce(&mut World) + Send + Sync + 'static,
    ) {
        self.on_success_error(
            move || OperationsPlugin::add(|w| s(w)),
            move || OperationsPlugin::add(|w| e(w)),
        );
    }
    fn notify_error(&self) {
        self.on_success(|| {});
    }
}

impl EventContext {
    pub fn check_identity(&self) -> bool {
        match &self.event {
            Event::Reducer(r) => r.caller_identity == player_identity(),
            _ => true,
        }
    }
}
