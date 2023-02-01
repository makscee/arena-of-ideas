use super::*;

pub struct BattleSystem {}

impl BattleSystem {
    pub fn tick(world: &mut legion::World, resources: &mut Resources) {
        let units = <(&UnitComponent, &EntityComponent, &Faction)>::query()
            .iter(world)
            .collect_vec();
        let left = units
            .iter()
            .find(|(_, _, faction)| **faction == Faction::Light);
        let right = units
            .iter()
            .find(|(_, _, faction)| **faction == Faction::Dark);
        if left.is_some() && right.is_some() && units.len() > 1 && resources.action_queue.is_empty()
        {
            let left = left.unwrap();
            let right = right.unwrap();
            let context = Context {
                owner: left.1.entity,
                target: right.1.entity,
                creator: left.1.entity,
                vars: default(),
                status: default(),
            };
            resources
                .action_queue
                .push_back(Action::new(context, Effect::Damage { value: None }));
            let context = Context {
                owner: right.1.entity,
                target: left.1.entity,
                creator: right.1.entity,
                vars: default(),
                status: default(),
            };
            resources
                .action_queue
                .push_back(Action::new(context, Effect::Damage { value: None }));
        }

        let dead_units = <(&EntityComponent, &HpComponent)>::query()
            .iter(world)
            .filter_map(|(unit, hp)| match hp.current <= 0 {
                true => Some(unit.entity),
                false => None,
            })
            .collect_vec();
        if !dead_units.is_empty() {
            dead_units.iter().for_each(|entity| {
                debug!("Entity#{:?} dead", entity);
                world.remove(*entity);
            });
        }
    }
}
