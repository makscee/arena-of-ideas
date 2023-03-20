use super::*;

pub struct ActionSystem {}

impl ActionSystem {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run_ticks(world: &mut legion::World, resources: &mut Resources) {
        let mut ticks = 0;
        while Self::tick(world, resources) && ticks < 1000 {
            ticks += 1;
        }
    }

    pub fn tick(world: &mut legion::World, resources: &mut Resources) -> bool {
        if resources.action_queue.is_empty() && resources.status_pool.status_changes.is_empty() {
            return false;
        }
        ContextSystem::refresh_all(world, resources);
        StatusPool::process_status_changes(world, resources);
        let Some(action) = resources.action_queue.pop_front() else { return false };
        resources.logger.log(
            &format!("Procession action: {:?}", action.effect),
            &LogContext::Action,
        );
        resources
            .logger
            .log(&format!("{:?}", action.context), &LogContext::Contexts);
        match action
            .effect
            .process(action.context.clone(), world, resources)
        {
            Ok(_) => {}
            Err(error) => {
                error!("Effect process error: {}", error);
                dbg!(action);
            }
        }
        ContextSystem::refresh_all(world, resources);
        true
    }
}

impl System for ActionSystem {
    fn update(&mut self, world: &mut legion::World, resources: &mut Resources) {
        Self::tick(world, resources);
    }
}

#[derive(Debug)]
pub struct Action {
    pub context: Context,
    pub effect: EffectWrapped,
}

impl Action {
    pub fn new(context: Context, effect: EffectWrapped) -> Self {
        Self { context, effect }
    }
}
