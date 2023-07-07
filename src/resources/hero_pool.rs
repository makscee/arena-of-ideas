use geng::prelude::itertools::Itertools;

use super::*;

#[derive(Default)]
pub struct HeroPool {
    heroes: HashMap<PathBuf, PackedUnit>,
    power: HashMap<String, usize>,
    list_top: PathBuf,
}

impl HeroPool {
    pub fn insert(path: PathBuf, unit: PackedUnit, resources: &mut Resources) {
        resources.hero_pool.heroes.insert(path, unit);
    }

    pub fn get<'a>(path: &PathBuf, resources: &'a Resources) -> &'a PackedUnit {
        resources.hero_pool.heroes.get(path).unwrap()
    }

    pub fn find_by_name<'a>(name: &str, resources: &'a Resources) -> Option<&'a PackedUnit> {
        resources
            .hero_pool
            .heroes
            .values()
            .find(|x| x.name.eq(name))
    }

    pub fn rarity_by_name(name: &str, resources: &Resources) -> Rarity {
        let ind = *resources.hero_pool.power.get(name).unwrap_or(&0);
        let perc = ind as f32 / resources.hero_pool.power.len() as f32;
        if perc > 0.9 {
            Rarity::Legendary
        } else if perc > 0.7 {
            Rarity::Epic
        } else if perc > 0.4 {
            Rarity::Rare
        } else {
            Rarity::Common
        }
    }

    pub fn all(resources: &Resources) -> Vec<PackedUnit> {
        resources.hero_pool.heroes.values().cloned().collect_vec()
    }

    pub fn names_sorted(resources: &Resources) -> Vec<String> {
        resources
            .hero_pool
            .power
            .iter()
            .sorted_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(name, _)| name.clone())
            .collect_vec()
    }

    pub fn len(resources: &Resources) -> usize {
        resources.hero_pool.heroes.len()
    }

    pub fn all_sorted(resources: &Resources) -> Vec<PackedUnit> {
        resources
            .hero_pool
            .heroes
            .values()
            .filter_map(|unit| {
                resources
                    .hero_pool
                    .power
                    .get(&unit.name)
                    .and_then(|x| Some((unit.clone(), x)))
            })
            .sorted_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|x| x.0)
            .collect_vec()
    }
}

impl FileWatcherLoader for HeroPool {
    fn load(resources: &mut Resources, path: &PathBuf, watcher: &mut FileWatcherSystem) {
        let mut path = path.clone();
        path.set_file_name("_list.json");
        watcher.watch_file(&path, Box::new(Self::load));
        let paths: Vec<PathBuf> = futures::executor::block_on(load_json(&path)).unwrap();
        resources.hero_pool.list_top = static_path().join(paths.get(0).unwrap());
        paths.into_iter().for_each(|path| {
            PackedUnit::load(resources, &static_path().join(path), watcher);
        });
        path.set_file_name("_rating.json");
        watcher.watch_file(&path, Box::new(Self::load));
        resources.hero_pool.power = futures::executor::block_on(load_json(path)).unwrap();
        for (ind, (_, power)) in resources
            .hero_pool
            .power
            .iter_mut()
            .sorted_by_key(|(_, rating)| **rating)
            .enumerate()
        {
            *power = ind;
        }
    }
}
