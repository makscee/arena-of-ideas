use geng::prelude::itertools::Itertools;

use super::*;

#[derive(Clone, Deserialize, Serialize, Debug, Default)]
pub struct BonusEffectPool {
    effects: Vec<BonusEffect>,
    #[serde(skip)]
    current: Vec<BonusEffect>,
}

impl BonusEffectPool {
    pub fn load_widget(world: &legion::World, resources: &mut Resources) {
        let entity = new_entity();
        let ts = resources.tape_player.head;
        Self::load_bonuses(world, resources);
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

    fn load_bonuses(world: &legion::World, resources: &mut Resources) {
        let mut current = resources
            .bonus_pool
            .effects
            .choose_multiple(&mut thread_rng(), 3)
            .cloned()
            .collect_vec();
        let units = UnitSystem::collect_faction(world, resources, Faction::Team, false);
        current.iter_mut().for_each(|x| {
            if x.single_target {
                let entity = *units.choose(&mut thread_rng()).unwrap();
                x.target = Some((entity, PackedUnit::pack(entity, world, resources)));
            }
        });
        resources.bonus_pool.current = current;
    }
}

impl FileWatcherLoader for BonusEffectPool {
    fn loader(resources: &mut Resources, path: &PathBuf, watcher: &mut FileWatcherSystem) {
        watcher.watch_file(path, Box::new(Self::loader));
        debug!("Load bonus effects pool {path:?}");
        resources.bonus_pool = futures::executor::block_on(load_json(path)).unwrap();
    }
}
