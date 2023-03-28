use geng::prelude::itertools::Itertools;

use super::*;

#[derive(Default)]
pub struct HeroPool {
    heroes: HashMap<PathBuf, PackedUnit>,
    power: HashMap<String, f32>,
}

impl HeroPool {
    pub fn insert(&mut self, path: PathBuf, unit: PackedUnit) {
        self.heroes.insert(path, unit);
    }

    pub fn get(&self, path: &PathBuf) -> &PackedUnit {
        self.heroes.get(path).unwrap()
    }

    pub fn all(&self) -> Vec<PackedUnit> {
        self.heroes.values().cloned().collect_vec()
    }

    pub fn all_sorted(&self) -> Vec<PackedUnit> {
        let mut heroes = self.heroes.values().cloned().collect_vec();
        heroes.sort_by(|a, b| {
            self.power
                .get(&a.name)
                .unwrap()
                .partial_cmp(self.power.get(&b.name).unwrap())
                .unwrap()
        });
        heroes
    }
}

impl FileWatcherLoader for HeroPool {
    fn loader(resources: &mut Resources, path: &PathBuf, watcher: &mut FileWatcherSystem) {
        watcher.watch_file(path, Box::new(Self::loader));
        let paths: Vec<PathBuf> =
            futures::executor::block_on(load_json(path.join("_list.json"))).unwrap();
        paths.into_iter().for_each(|path| {
            PackedUnit::loader(resources, &static_path().join(path), watcher);
        });
        resources.hero_pool.power =
            futures::executor::block_on(load_json(path.join("_power.json"))).unwrap();
    }
}
