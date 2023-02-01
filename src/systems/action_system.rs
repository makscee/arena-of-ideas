use super::*;

pub struct ActionSystem {}

impl ActionSystem {
    pub fn new() -> Self {
        Self {}
    }
}

impl System for ActionSystem {
    fn update(&mut self, world: &mut legion::World, resources: &mut Resources) {
        let Some(action) = resources.action_queue.pop_front() else { return };
        debug!(
            "Procession action: {:?} context: {:?}",
            action.effect, action.context
        );
        match action
            .effect
            .process(action.context.clone(), world, resources)
        {
            Ok(_) => {}
            Err(error) => error!("Effect proectss error: {}", error),
        }
    }

    fn draw(
        &self,
        _world: &legion::World,
        _resources: &Resources,
        _framebuffer: &mut ugli::Framebuffer,
    ) {
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
