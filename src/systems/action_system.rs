use super::*;

pub struct ActionSystem {}

impl ActionSystem {
    pub fn run_ticks(
        world: &mut legion::World,
        resources: &mut Resources,
        mut cluster: Option<&mut NodeCluster>,
    ) {
        let mut ticks = 0;
        loop {
            let ticked = if let Some(node_cluster) = cluster {
                let node = &mut Some(Node::default());
                let result = Self::tick(world, resources, node);
                node_cluster.push(
                    node.take()
                        .unwrap()
                        .lock(NodeLockType::Full { world, resources }),
                );
                cluster = Some(node_cluster);
                result
            } else {
                Self::tick(world, resources, &mut None)
            };
            if !ticked {
                break;
            }
            ticks += 1;
            if ticks > 1000 {
                let dark_name = TeamSystem::get_state(Faction::Dark, world).name.clone();
                let light_name = TeamSystem::get_state(Faction::Light, world).name.clone();
                panic!("Exceeded ticks limit {light_name} vs. {dark_name}")
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
        mut cluster: Option<&mut NodeCluster>,
    ) {
        Self::run_ticks(world, resources, cluster.as_deref_mut());
        Self::death_check(world, resources, cluster);
    }

    pub fn death_check(
        world: &mut legion::World,
        resources: &mut Resources,
        mut cluster: Option<&mut NodeCluster>,
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
            if let Some(killer) =
                UnitSystem::process_death(dead_unit, world, resources, cluster.as_deref_mut())
            {
                resources.logger.log(
                    || format!("{:?} removed", dead_unit),
                    &LogContext::UnitCreation,
                );
                corpses.push((dead_unit, killer));
            }
        }
        for (entity, killer) in corpses {
            Event::UnitDeath {
                target: entity,
                killer,
            }
            .send(world, resources);
            if killer != entity && UnitSystem::is_alive(killer, world, resources) {
                Event::AfterKill {
                    owner: killer,
                    target: entity,
                }
                .send(world, resources);
            }
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
