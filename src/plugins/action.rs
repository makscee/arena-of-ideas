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
                action.invoke(world);
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

    pub fn push_back(effect: EffectWrapped, context: Context, world: &mut World) {
        debug!("Push back {:?}", effect.effect);
        let action = Action { context, effect };
        world
            .get_resource_mut::<ActionQueue>()
            .unwrap()
            .push_back(action);
    }

    pub fn push_front(effect: EffectWrapped, context: Context, world: &mut World) {
        debug!("Push front {:?}", effect.effect);
        let action = Action { context, effect };
        world
            .get_resource_mut::<ActionQueue>()
            .unwrap()
            .push_front(action);
    }
}

#[derive(Debug)]
pub struct Action {
    pub context: Context,
    pub effect: EffectWrapped,
}

impl Action {
    pub fn invoke(mut self, world: &mut World) {
        match self.effect.invoke(&mut self.context, world) {
            Ok(_) => {
                for entity in world
                    .query_filtered::<Entity, (With<Status>, With<VarStateDelta>, With<Parent>)>()
                    .iter(world)
                    .collect_vec()
                {
                    Status::apply_delta(entity, world);
                }
            }
            Err(err) => error!(
                "Effect process error {err}\n{}\n---\n{}",
                self.effect.effect, self.context
            ),
        }
    }
}

#[derive(Resource, Default)]
pub struct ActionQueue(VecDeque<Action>);

impl ActionQueue {
    pub fn push_back(&mut self, action: Action) {
        self.0.push_back(action)
    }
    pub fn push_front(&mut self, action: Action) {
        self.0.push_front(action)
    }
}
