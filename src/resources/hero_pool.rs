use geng::prelude::itertools::Itertools;

use super::*;

#[derive(Default)]
pub struct HeroPool {
    heroes: HashMap<PathBuf, PackedUnit>,
    power: HashMap<String, f32>,
    list_top: PathBuf,
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

    pub fn list_top(&self) -> &PackedUnit {
        self.heroes.get(&self.list_top).unwrap()
    }

    pub fn all_sorted(&self) -> Vec<PackedUnit> {
        self.heroes
            .values()
            .filter_map(|unit| {
                self.power
                    .get(&unit.name)
                    .and_then(|x| Some((unit.clone(), x)))
            })
            .sorted_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|x| x.0)
            .collect_vec()
    }
}

impl FileWatcherLoader for HeroPool {
    fn loader(resources: &mut Resources, path: &PathBuf, watcher: &mut FileWatcherSystem) {
        let mut path = path.clone();
        path.set_file_name("_list.json");
        watcher.watch_file(&path, Box::new(Self::loader));
        let paths: Vec<PathBuf> = futures::executor::block_on(load_json(&path)).unwrap();
        resources.hero_pool.list_top = static_path().join(paths.get(0).unwrap());
        paths.into_iter().for_each(|path| {
            PackedUnit::loader(resources, &static_path().join(path), watcher);
        });
        path.set_file_name("_power.json");
        watcher.watch_file(&path, Box::new(Self::loader));
        resources.hero_pool.power = futures::executor::block_on(load_json(path)).unwrap();
    }
}
