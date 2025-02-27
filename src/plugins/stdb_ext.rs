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
            Event::UnknownTransaction => "Unknown transaction".notify_error_op(),
            _ => panic!(),
        }
    }
    fn on_success_op(&self, f: impl FnOnce(&mut World) + Send + Sync + 'static) {
        self.on_success(move || OperationsPlugin::add(|w| f(w)));
    }
    fn notify_error(&self) {
        match self {
            Event::Reducer(r) => match &r.status {
                spacetimedb_sdk::Status::Committed => {}
                spacetimedb_sdk::Status::Failed(e) => e.notify_error_op(),
                _ => panic!(),
            },
            Event::SubscribeError(e) => e.to_string().notify_error_op(),
            Event::UnknownTransaction => "Unknown transaction".notify_error_op(),
            _ => panic!(),
        }
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
