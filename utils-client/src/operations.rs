use super::*;

use bevy::prelude::*;
use once_cell::sync::OnceCell;
use std::collections::VecDeque;

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
    pub fn add(operation: impl FnOnce(&mut World) + Send + Sync + 'static) {
        let operation = Box::new(operation);
        Self::add_boxed(operation)
    }
    pub fn add_boxed(operation: Box<impl FnOnce(&mut World) + Send + Sync + 'static>) {
        OPERATIONS.get().unwrap().lock().queue.push(operation)
    }
}

pub fn op(f: impl FnOnce(&mut World) + 'static + Send + Sync) {
    OperationsPlugin::add(f);
}

fn update(world: &mut World) {
    let v = std::mem::take(&mut OPERATIONS.get().unwrap().lock().queue);
    for operation in v {
        operation(world);
    }
}
