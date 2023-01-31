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
            Ok(context) => {
                if let Some(mut target) = world.entry(context.target) {
                    target
                        .get_component_mut::<Context>()
                        .expect(&format!(
                            "No Context component on Target#{:?}",
                            context.target
                        ))
                        .vars
                        .update_self(&context.vars);
                }
                if let Some((name, entity)) = context.status {
                    if let Some(status_context) = resources
                        .statuses
                        .active_statuses
                        .get_mut(&entity)
                        .and_then(|entry| entry.get_mut(&name))
                    {
                        status_context.vars.override_self(&context.vars);
                    }
                }
            }
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
