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
        self.on_success(move || op(|w| f(w)));
    }
    fn on_success_error(
        &self,
        sfn: impl FnOnce() + Send + Sync + 'static,
        efn: impl FnOnce() + Send + Sync + 'static,
    ) {
        match &self.status {
            spacetimedb_sdk::Status::Committed => sfn(),
            spacetimedb_sdk::Status::Failed(e) => {
                format!("STDB error: {e}").notify_error_op();
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
        self.on_success_error(move || op(|w| s(w)), move || op(|w| e(w)));
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
    fn entity(self, ctx: &ClientContext) -> NodeResult<Entity>;
    fn get_node(self) -> Option<TNode>;
    fn kind_db(self) -> Result<NodeKind, NodeError>;
    fn label(self, ui: &mut Ui) -> Response;
    fn node_rating(self) -> Option<i32>;
    fn fixed_kinds(self) -> HashSet<NodeKind>;
}

impl NodeIdExt for u64 {
    fn entity(self, ctx: &ClientContext) -> NodeResult<Entity> {
        ctx.entity(self)
    }
    fn get_node(self) -> Option<TNode> {
        cn().db.nodes_world().id().find(&self)
    }
    fn kind_db(self) -> Result<NodeKind, NodeError> {
        Ok(cn()
            .db
            .nodes_world()
            .id()
            .find(&self)
            .to_e_not_found()?
            .kind())
    }
    fn label(self, ui: &mut Ui) -> Response {
        let id = self;
        let resp = format!("[s [tw #]{}]", id % 100000)
            .label(ui)
            .on_hover_ui(|ui| {
                format!("[tw #]{id}").label(ui);
            });
        if resp.clicked() {
            clipboard_set(id);
        }
        resp
    }
    fn node_rating(self) -> Option<i32> {
        cn().db.nodes_world().id().find(&self).map(|n| n.rating)
    }
    fn fixed_kinds(self) -> HashSet<NodeKind> {
        let id: u64 = self;
        let kinds = cn()
            .db
            .creation_phases()
            .node_id()
            .find(&id)
            .map(|cp| cp.fixed_kinds)
            .unwrap_or_default();
        HashSet::from_iter(kinds.into_iter().map(|k| k.to_kind()))
    }
}
