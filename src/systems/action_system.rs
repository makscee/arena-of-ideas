use super::*;

pub struct ActionSystem {}

impl ActionSystem {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run_ticks(world: &mut legion::World, resources: &mut Resources) -> Vec<CassetteNode> {
        let mut ticks = 0;
        let mut nodes = Vec::default();
        loop {
            let (ticked, node) = Self::tick(world, resources);
            ticks += 1;
            if let Some(node) = node {
                nodes.push(node);
            }
            if !ticked || ticks > 1000 {
                if ticked {
                    panic!("Exceeded ticks limit")
                }
                break;
            }
        }
        nodes
    }

    pub fn tick(
        world: &mut legion::World,
        resources: &mut Resources,
    ) -> (bool, Option<CassetteNode>) {
        if !resources.status_pool.status_changes.is_empty() {
            ContextSystem::refresh_all(world, resources);
            let node = StatusPool::process_status_changes(world, resources);
            return (true, node);
        } else if !resources.action_queue.is_empty() {
            ContextSystem::refresh_all(world, resources);
            let action = resources.action_queue.pop_front().unwrap();
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
                Ok(node) => {
                    return (true, Some(node));
                }
                Err(error) => {
                    error!("Effect process error: {}", error);
                    dbg!(action);
                }
            }
        }
        (false, None)
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
