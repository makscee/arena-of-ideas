use spacetimedb_sdk::{ReducerEvent, Table};

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

impl<R> StdbStatusExt for ReducerEvent<R> {
    fn on_success(&self, f: impl FnOnce() + Send + Sync + 'static) {
        match &self.status {
            spacetimedb_sdk::Status::Committed => f(),
            spacetimedb_sdk::Status::Failed(e) => e.notify_error_op(),
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
        match &self.status {
            spacetimedb_sdk::Status::Committed => sfn(),
            spacetimedb_sdk::Status::Failed(e) => {
                e.notify_error_op();
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

impl ReducerEventContext {
    pub fn check_identity(&self) -> bool {
        self.event.caller_identity == player_identity()
    }
}

pub trait NodeIdExt {
    fn get_node(self) -> Option<TNode>;
    fn kind(self) -> Result<NodeKind, NodeError>;
    fn label(self, ui: &mut Ui) -> Response;
    fn node_rating(self) -> Option<i32>;
}

impl NodeIdExt for u64 {
    fn get_node(self) -> Option<TNode> {
        cn().db.nodes_world().id().find(&self)
    }
    fn kind(self) -> Result<NodeKind, NodeError> {
        Ok(cn()
            .db
            .nodes_world()
            .id()
            .find(&self)
            .to_e_not_found()?
            .kind())
    }
    fn label(self, ui: &mut Ui) -> Response {
        format!("[s [tw #]{}]", self % 100000)
            .label(ui)
            .on_hover_ui(|ui| {
                format!("[tw #]{self}").label(ui);
            })
    }
    fn node_rating(self) -> Option<i32> {
        cn().db.nodes_world().id().find(&self).map(|n| n.rating)
    }
}
