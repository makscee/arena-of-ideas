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
        let ts = resources.tape_player.head;
        Self::load_bonuses(value, world, resources);
        let bonuses = resources.bonus_pool.current.clone();
        let panel = Widget::BonusChoicePanel { bonuses, entity }.generate_node(&resources.options);
        let panel = NodePanel::new(panel, ts);
        resources.tape_player.tape.panels.insert(entity, panel);
    }

    pub fn make_selection(ind: usize, world: &legion::World, resources: &mut Resources) {
        let pool = &mut resources.bonus_pool;
        let bonus = pool.current[ind].to_owned();
        pool.current.clear();
        let mut context = WorldSystem::get_context(world);
        if let Some((target, _)) = bonus.target {
            context.target = target;
        }
        resources
            .action_queue
            .push_back(Action::new(context, bonus.effect));
    }

    fn load_bonuses(value: usize, world: &legion::World, resources: &mut Resources) {
        let units = UnitSystem::collect_faction(world, resources, Faction::Team, false);
        resources.bonus_pool.current.clear();

        let all_rarities = enum_iterator::all::<Rarity>().collect_vec();
        let mut rarities: HashSet<Rarity> = default();
        for (i, rarity) in enum_iterator::all::<Rarity>().enumerate() {
            if i >= value {
                break;
            }
            rarities.insert(rarity);
        }

        let option_count = 3 + (value as i32 - all_rarities.len() as i32).max(0) as usize;
        let bonuses = resources
            .bonus_pool
            .effects
            .iter()
            .filter_map(|(rarity, bonuses)| match rarities.contains(rarity) {
                true => Some(bonuses),
                false => None,
            })
            .flatten()
            .filter(|x| !units.is_empty() || !x.single_target)
            .cloned()
            .collect_vec();

        let mut current = bonuses
            .into_iter()
            .choose_multiple(&mut thread_rng(), option_count);

        current.iter_mut().for_each(|x| {
            if x.single_target {
                let entity = *units.choose(&mut thread_rng()).unwrap();
                x.target = Some((entity, UnitSystem::unit_string(entity, world)));
            }
        });
        resources.bonus_pool.current = current;
    }
}

impl FileWatcherLoader for BonusEffectPool {
    fn loader(resources: &mut Resources, path: &PathBuf, watcher: &mut FileWatcherSystem) {
        watcher.watch_file(path, Box::new(Self::loader));
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
