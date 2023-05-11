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

    pub fn spin(
        world: &mut legion::World,
        resources: &mut Resources,
        cluster: &mut Option<NodeCluster>,
    ) {
        Self::run_ticks(world, resources, cluster);
        Self::death_check(world, resources, cluster);
    }

    pub fn death_check(
        world: &mut legion::World,
        resources: &mut Resources,
        cluster: &mut Option<NodeCluster>,
    ) {
        let mut corpses = Vec::default();
        while let Some(dead_unit) = <&EntityComponent>::query()
            .filter(component::<UnitComponent>())
            .iter(world)
            .filter_map(
                |entity| match UnitSystem::is_alive(entity.entity, world, resources) {
                    false => Some(entity.entity),
                    true => None,
                },
            )
            .choose(&mut thread_rng())
        {
            resources.logger.log(
                || format!("{:?} dead", dead_unit),
                &LogContext::UnitCreation,
            );
            if UnitSystem::process_death(dead_unit, world, resources, cluster) {
                resources.logger.log(
                    || format!("{:?} removed", dead_unit),
                    &LogContext::UnitCreation,
                );
                corpses.push(dead_unit);
            }
        }
        for entity in corpses {
            Event::UnitDeath { target: entity }.send(world, resources);
        }
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
