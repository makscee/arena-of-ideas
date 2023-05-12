use geng::prelude::itertools::Itertools;

use super::*;

#[derive(Clone, Deserialize, Serialize, Debug, Default)]
pub struct BonusEffectPool {
    effects: HashMap<Rarity, Vec<BonusEffect>>,
    #[serde(skip)]
    current: Vec<BonusEffect>,
}

impl BonusEffectPool {
    pub fn load_widget(value: usize, world: &legion::World, resources: &mut Resources) {
        debug!("Load bonus choice widget {value}");
        let entity = new_entity();
        Self::load_bonuses(value, world, resources);
        let bonuses = resources.bonus_pool.current.clone();
        Widget::BonusChoicePanel {
            bonuses,
            panel_entity: entity,
            options: &resources.options,
            value,
        }
        .generate_node()
        .lock(NodeLockType::Empty)
        .push_as_panel(entity, resources);
    }

    pub fn make_selection(ind: usize, world: &legion::World, resources: &mut Resources) {
        let pool = &mut resources.bonus_pool;
        if pool.current.is_empty() {
            return;
        }
        let bonus = pool.current[ind].to_owned();
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
    }

    fn load_bonuses(value: usize, world: &legion::World, resources: &mut Resources) {
        let units = UnitSystem::collect_faction(world, Faction::Team);
        resources.bonus_pool.current.clear();

        let option_count = (1 + value).min(4);
        let all_rarities = enum_iterator::all::<Rarity>().collect_vec();
        let mut rarities = all_rarities
            .choose_multiple_weighted(&mut thread_rng(), option_count, |item| item.weight())
            .unwrap()
            .collect_vec();
        let ind = (&mut thread_rng()).gen_range(0..rarities.len());
        rarities[ind] = &all_rarities[(value).min(all_rarities.len()) - 1];

        let mut current = rarities
            .into_iter()
            .map(|x| x.generate(value, units.len(), resources))
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
