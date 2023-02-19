use super::*;

pub struct ActionSystem {}

impl ActionSystem {
    pub fn new() -> Self {
        Self {}
    }

    pub fn tick(world: &mut legion::World, resources: &mut Resources) -> bool {
        let Some(mut action) = resources.action_queue.pop_front() else { return false };
        action
            .context
            .vars
            .merge(WorldSystem::get_vars(world), false);
        debug!(
            "Procession action: {:?} context: {:?}",
            action.effect, action.context
        );
        match action
            .effect
            .process(action.context.clone(), world, resources)
        {
            Ok(_) => {}
            Err(error) => error!("Effect process error: {}", error),
        }
        true
    }
}

impl System for ActionSystem {
    fn update(&mut self, world: &mut legion::World, resources: &mut Resources) {
        Self::tick(world, resources);
    }
}

pub struct Action {
    pub context: Context,
    pub effect: Effect,
}

impl Action {
    pub fn new(context: Context, effect: Effect) -> Self {
        Self { context, effect }
    }
}
