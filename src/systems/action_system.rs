use super::*;

pub struct ActionSystem {}

impl ActionSystem {
    pub fn run_ticks(
        world: &mut legion::World,
        resources: &mut Resources,
        nodes: &mut Option<Vec<CassetteNode>>,
    ) {
        let mut ticks = 0;
        loop {
            let ticked = if let Some(nodes) = nodes {
                let mut node = CassetteNode::default();
                node.skip_part = 0.5;
                let node = &mut Some(node);
                let result = Self::tick(world, resources, node);
                nodes.push(node.take().unwrap().finish(world, resources));
                result
            } else {
                Self::tick(world, resources, &mut None)
            };
            if !ticked {
                if ticks > 0 {
                    if let Some(nodes) = nodes {
                        nodes.last_mut().unwrap().skip_part = 0.0;
                    }
                }
                break;
            }
            ticks += 1;
            if ticks > 1000 {
                panic!("Exceeded ticks limit")
            }
        }
    }

    pub fn tick(
        world: &mut legion::World,
        resources: &mut Resources,
        node: &mut Option<CassetteNode>,
    ) -> bool {
        let result = if !resources.status_pool.status_changes.is_empty() {
            ContextSystem::refresh_all(world, resources);
            StatusPool::process_status_changes(world, resources, node);
            true
        } else if !resources.action_queue.is_empty() {
            ContextSystem::refresh_all(world, resources);
            let action = resources.action_queue.pop_front().unwrap();
            resources.logger.log(
                &format!(
                    "Process t:{:?} o:{:?}: {:?}",
                    action.context.target, action.context.owner, action.effect
                ),
                &LogContext::Action,
            );
            resources
                .logger
                .log(&format!("{:?}", action.context), &LogContext::Contexts);
            match action
                .effect
                .process(action.context.clone(), world, resources, node)
            {
                Ok(_) => {}
                Err(error) => {
                    resources
                        .logger
                        .log(&format!("{}", error), &LogContext::ActionFail);
                }
            };
            true
        } else {
            false
        };
        result
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
