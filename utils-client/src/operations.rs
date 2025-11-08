use super::*;

use bevy::prelude::*;
use once_cell::sync::OnceCell;

pub type Operation = Box<dyn FnOnce(&mut World) + Send + Sync>;
pub struct OperationsPlugin;

static OPERATIONS: OnceCell<Mutex<OperationsData>> = OnceCell::new();

#[derive(Default)]
pub struct OperationsData {
    queue: Vec<Operation>,
}

impl Plugin for OperationsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, || {
            let _ = OPERATIONS.set(Mutex::new(default()));
        })
        .add_systems(Update, update);
    }
}

impl OperationsPlugin {
    pub fn add(back: bool, operation: impl FnOnce(&mut World) + Send + Sync + 'static) {
        let operation = Box::new(operation);
        Self::add_boxed(back, operation)
    }
    pub fn add_boxed(back: bool, operation: Box<impl FnOnce(&mut World) + Send + Sync + 'static>) {
        let q = &mut OPERATIONS.get().unwrap().lock().queue;
        if back {
            q.push(operation);
        } else {
            q.insert(0, operation);
        }
    }
}

pub fn op(f: impl FnOnce(&mut World) + 'static + Send + Sync) {
    OperationsPlugin::add(true, f);
}
pub fn op_front(f: impl FnOnce(&mut World) + 'static + Send + Sync) {
    OperationsPlugin::add(false, f);
}

fn update(world: &mut World) {
    let v = std::mem::take(&mut OPERATIONS.get().unwrap().lock().queue);
    for operation in v {
        operation(world);
    }
}
