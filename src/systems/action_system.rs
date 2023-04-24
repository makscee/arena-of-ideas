use super::*;

pub struct ActionSystem {}

impl ActionSystem {
    pub fn run_ticks(
        world: &mut legion::World,
        resources: &mut Resources,
        cluster: &mut Option<NodeCluster>,
    ) {
        let mut ticks = 0;
        loop {
            let ticked = if let Some(cluster) = cluster {
                let node = &mut Some(Node::default());
                let result = Self::tick(world, resources, node);
                cluster.push(
                    node.take()
                        .unwrap()
                        .lock(NodeLockType::Full { world, resources }),
                );
                result
            } else {
                Self::tick(world, resources, &mut None)
            };
            if !ticked {
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
        node: &mut Option<Node>,
    ) -> bool {
        let result = if !resources.action_queue.is_empty() {
            let action = resources.action_queue.pop_front().unwrap();
            let context = action.context;
            let effect = action.effect;

            resources.logger.log(
                || {
                    format!(
                        "Process t:{:?} o:{:?}: {:?}",
                        context.target(),
                        context.owner(),
                        effect
                    )
                },
                &LogContext::Action,
            );
            resources
                .logger
                .log(|| format!("{:?}", context), &LogContext::Contexts);
            match effect.process(context, world, resources, node) {
                Ok(_) => {}
                Err(error) => {
                    resources
                        .logger
                        .log(|| error.to_string(), &LogContext::ActionFail);
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
