use spacetimedb_sdk::reducer::Status;

use self::module_bindings::{
    once_on_run_buy, once_on_run_fuse, once_on_run_reroll, once_on_run_sell, once_on_run_stack,
    once_on_run_submit_result, once_on_upload_units, run_buy, run_fuse, run_reroll, run_sell,
    run_stack, run_submit_result, upload_units, TableUnit,
};

use super::*;

pub struct ServerPlugin;

impl ServerPlugin {
    pub fn pending(world: &World) -> Option<&PendingOperation> {
        world.get_resource::<PendingOperation>()
    }

    fn store_pending_operation(operation: ServerOperation, world: &mut World) {
        world.insert_resource(PendingOperation(operation));
    }
}

#[derive(Resource, Debug)]
pub struct PendingOperation(ServerOperation);

#[derive(Clone, Debug)]
#[must_use]
pub enum ServerOperation {
    Sell(Entity),
    Buy(Entity),
    Stack {
        target: Entity,
        source: Entity,
    },
    Fuse {
        a: Entity,
        b: Entity,
        fused: PackedUnit,
    },
    Reroll,
    SubmitResult(bool),
    UploadUnits(Vec<PackedUnit>),
}

fn clear_pending(status: &Status) {
    match status {
        Status::Failed(e) => AlertPlugin::add_error(
            Some("Server operation error".to_owned()),
            e.to_owned(),
            None,
        ),
        Status::Committed => info!("Server operation commited"),
        _ => {}
    }
    OperationsPlugin::add(move |world| {
        world.remove_resource::<PendingOperation>();
    });
    info!("Server operation finished {status:?}");
}

impl ServerOperation {
    pub fn send(self, world: &mut World) -> Result<()> {
        info!("Send server operation {self:?}");
        if let Some(o) = ServerPlugin::pending(world) {
            return Err(anyhow!("Operation pending already {o:?}"));
        }
        ServerPlugin::store_pending_operation(self.clone(), world);
        match self {
            ServerOperation::Sell(entity) => {
                let id = UnitPlugin::get_id(entity, world).context("Id not found")?;
                run_sell(id);
                once_on_run_sell(|_, _, status, _| clear_pending(status));
            }
            ServerOperation::Buy(entity) => {
                let id = UnitPlugin::get_id(entity, world).context("Id not found")?;
                run_buy(id);
                once_on_run_buy(|_, _, status, _| clear_pending(status));
            }
            ServerOperation::Stack { target, source } => {
                let target = UnitPlugin::get_id(target, world).context("Id not found")?;
                let source = UnitPlugin::get_id(source, world).context("Id not found")?;
                run_stack(target, source);
                once_on_run_stack(|_, _, status, target, _| {
                    clear_pending(status);
                    if matches!(status, Status::Committed) {
                        let target = *target;
                        OperationsPlugin::add(move |world| {
                            if let Some(target) = UnitPlugin::get_by_id(target, world) {
                                TextColumn::add(target, "+Stack", yellow(), world).unwrap();
                            }
                        })
                    }
                });
            }
            ServerOperation::Fuse { a, b, fused } => {
                let a = UnitPlugin::get_id(a, world).context("Id not found")?;
                let b = UnitPlugin::get_id(b, world).context("Id not found")?;
                run_fuse(a, b, fused.into());
                once_on_run_fuse(|_, _, status, _, _, _| clear_pending(status));
            }
            ServerOperation::Reroll => {
                run_reroll(false);
                once_on_run_reroll(|_, _, status, _| clear_pending(status));
            }
            ServerOperation::SubmitResult(win) => {
                run_submit_result(win);
                once_on_run_submit_result(|_, _, status, _| clear_pending(status));
            }
            ServerOperation::UploadUnits(units) => {
                let units: Vec<TableUnit> = units.into_iter().map(|u| u.into()).collect_vec();
                info!("Upload {} units start", units.len());
                upload_units(units);
                once_on_upload_units(|_, _, status, _| clear_pending(status));
            }
        };
        Ok(())
    }
}
