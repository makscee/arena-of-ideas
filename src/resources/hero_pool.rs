use geng::prelude::itertools::Itertools;

use super::*;

#[derive(Default)]
pub struct HeroPool(HashMap<PathBuf, PackedUnit>);

impl HeroPool {
    pub fn insert(&mut self, path: PathBuf, unit: PackedUnit) {
        self.0.insert(path, unit);
    }

    pub fn get(&self, path: &PathBuf) -> &PackedUnit {
        self.0.get(path).unwrap()
    }

    pub fn all(&self) -> Vec<PackedUnit> {
        self.0.values().cloned().collect_vec()
    }
}

impl FileWatcherLoader for HeroPool {
    fn loader(resources: &mut Resources, path: &PathBuf, watcher: &mut FileWatcherSystem) {
        watcher.watch_file(path, Box::new(Self::loader));
        let paths: Vec<PathBuf> = futures::executor::block_on(load_json(path)).unwrap();
        paths.into_iter().for_each(|path| {
            PackedUnit::loader(resources, &static_path().join(path), watcher);
        })
    }
}
