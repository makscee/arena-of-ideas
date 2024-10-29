use std::collections::VecDeque;

use super::*;

type Operation = Box<dyn FnOnce(&mut World) + Send + Sync>;
pub struct OperationsPlugin;

lazy_static! {
    static ref OPERATIONS: Mutex<OperationsData> = Mutex::new(default());
}

#[derive(Default)]
pub struct OperationsData {
    queue: VecDeque<Operation>,
}

impl Plugin for OperationsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update);
    }
}

impl OperationsPlugin {
    pub fn add(operation: impl FnOnce(&mut World) + Send + Sync + 'static) {
        let operation = Box::new(operation);
        Self::add_boxed(operation)
    }
    pub fn add_boxed(operation: Box<impl FnOnce(&mut World) + Send + Sync + 'static>) {
        OPERATIONS.lock().unwrap().queue.push_back(operation)
    }
}

fn update(world: &mut World) {
    while let Some(operation) = OPERATIONS.lock().unwrap().queue.pop_front() {
        operation(world);
    }
}
