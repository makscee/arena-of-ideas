use std::collections::VecDeque;

use super::*;

pub struct ActionPlugin;

impl Plugin for ActionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActionQueue>()
            .add_systems(Update, Self::process_queue);
    }
}

impl ActionPlugin {
    fn process_queue(world: &mut World) {
        if let Some(mut queue) = world.get_resource_mut::<ActionQueue>() {
            if let Some(action) = queue.0.pop_front() {
                action.process(world);
            }
        }
    }
}

#[derive(Debug)]
pub struct Action {
    pub context: Context,
    pub effect: Effect,
}

impl Action {
    pub fn process(self, world: &mut World) {
        self.effect.process(self.context, world);
    }
}

#[derive(Resource, Default)]
pub struct ActionQueue(VecDeque<Action>);

impl ActionQueue {
    pub fn push(&mut self, action: Action) {
        self.0.push_back(action)
    }
}
