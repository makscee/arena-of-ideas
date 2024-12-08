use std::{collections::VecDeque, sync::Mutex};

use once_cell::sync::OnceCell;

use super::*;

type Operation = Box<dyn FnOnce(&mut World) + Send + Sync>;
pub struct OperationsPlugin;

static OPERATIONS: OnceCell<Mutex<OperationsData>> = OnceCell::new();

#[derive(Default)]
pub struct OperationsData {
    queue: VecDeque<Operation>,
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
        OPERATIONS
            .get()
            .unwrap()
            .lock()
            .unwrap()
            .queue
            .push_back(operation)
    }
}

fn update(world: &mut World) {
    while let Some(operation) = OPERATIONS.get().unwrap().lock().unwrap().queue.pop_front() {
        operation(world);
    }
}
