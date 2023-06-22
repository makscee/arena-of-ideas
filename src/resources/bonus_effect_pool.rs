use geng::prelude::itertools::Itertools;

use super::*;

#[derive(Clone, Default)]
pub struct BonusEffectPool {
    effects: HashMap<Rarity, Vec<BonusEffect>>,
    current: Vec<BonusEffect>,
    pub after: Option<fn(&mut legion::World, &mut Resources)>,
}

impl BonusEffectPool {
    pub fn make_selection(ind: usize, world: &mut legion::World, resources: &mut Resources) {
        let pool = &mut resources.bonus_pool;
        if pool.current.is_empty() {
            return;
        }
        let bonus = pool.current[ind].to_owned();
        let after = pool.after.take();
        pool.current.clear();
        let mut context = Context::new(
            ContextLayer::Entity {
                entity: WorldSystem::entity(world),
            },
            world,
            resources,
        );
        if let Some((target, _)) = bonus.target {
            context.stack(ContextLayer::Target { entity: target }, world, resources);
        }
        resources
            .action_queue
            .push_back(Action::new(context, bonus.effect));
        if let Some(after) = after {
            (after)(world, resources);
        }
    }

    fn load_bonuses(value: usize, world: &legion::World, resources: &mut Resources) {
        let units = UnitSystem::collect_faction(world, Faction::Team);
        resources.bonus_pool.current.clear();

        let option_count = (value.saturating_sub(2)).clamp(2, 4);
        let all_rarities = enum_iterator::all::<Rarity>().collect_vec();
        let rarities = (0..option_count)
            .map(|_| {
                all_rarities
                    .choose_weighted(&mut thread_rng(), |i| i.weight())
                    .unwrap()
            })
            .collect_vec();
        // rarities[ind] = &all_rarities[(value).min(all_rarities.len()) - 1];

        let ind = if TeamSystem::get_state(&Faction::Team, world).get_int(&VarName::Slots, world)
            < MAX_SLOTS as i32
        {
            (&mut thread_rng()).gen_range(0..rarities.len())
        } else {
            MAX_SLOTS + 1
        };
        let mut current = rarities
            .into_iter()
            .enumerate()
            .map(|(i, x)| x.generate(i != ind, resources))
            .collect_vec();

        current.iter_mut().for_each(|x| {
            if x.single_target {
                let entity = *units.choose(&mut thread_rng()).unwrap();
                x.target = Some((entity, UnitSystem::unit_string(entity, world, resources)));
            }
        });
        resources.bonus_pool.current = current;
    }
}

impl FileWatcherLoader for BonusEffectPool {
    fn load(resources: &mut Resources, path: &PathBuf, watcher: &mut FileWatcherSystem) {
        watcher.watch_file(path, Box::new(Self::load));
        debug!("Load bonus effects pool {path:?}");
        resources.bonus_pool.effects =
            HashMap::from_iter(enum_iterator::all::<Rarity>().map(|x| (x, default())));
        let effects: Vec<BonusEffect> = futures::executor::block_on(load_json(path)).unwrap();
        for effect in effects {
            resources
                .bonus_pool
                .effects
                .get_mut(&effect.rarity)
                .unwrap()
                .push(effect);
        }
    }
}
