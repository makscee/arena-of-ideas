use std::collections::VecDeque;

use super::*;

pub struct ActionPlugin;

impl Plugin for ActionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActionQueue>()
            .add_systems(Update, Self::process_checker);
    }
}

impl ActionPlugin {
    fn process_queue(world: &mut World) -> bool {
        if let Some(mut queue) = world.get_resource_mut::<ActionQueue>() {
            if let Some(action) = queue.0.pop_front() {
                action.process(world);
                return true;
            }
        }
        false
    }

    fn process_checker(world: &mut World) {
        Self::process_queue(world);
    }

    pub fn spin(world: &mut World) {
        loop {
            if !Self::process_queue(world) {
                break;
            }
        }
    }

    pub fn queue_effect(effect: Effect, context: Context, world: &mut World) {
        let action = Action { context, effect };
        world
            .get_resource_mut::<ActionQueue>()
            .unwrap()
            .push(action);
    }
}

#[derive(Debug)]
pub struct Action {
    pub context: Context,
    pub effect: Effect,
}

impl Action {
    pub fn process(self, world: &mut World) {
        match self.effect.process(self.context, world) {
            Ok(_) => {}
            Err(err) => error!("Effect process error {err}"),
        }
    }
}

#[derive(Resource, Default)]
pub struct ActionQueue(VecDeque<Action>);

impl ActionQueue {
    pub fn push(&mut self, action: Action) {
        self.0.push_back(action)
    }
}
