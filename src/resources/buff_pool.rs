use geng::prelude::itertools::Itertools;

use super::*;

#[derive(Deserialize, Debug, Default)]
pub struct BuffPool {
    pool: Vec<Buff>,
}
impl BuffPool {
    pub fn get_random(count: usize, resources: &Resources) -> Vec<Buff> {
        resources
            .buff_pool
            .pool
            .choose_multiple_weighted(&mut thread_rng(), count, |x| x.rarity.weight())
            .unwrap()
            .cloned()
            .collect_vec()
    }

    pub fn get_by_name(name: &str, resources: &Resources) -> Buff {
        resources
            .buff_pool
            .pool
            .iter()
            .find(|x| x.name == name)
            .unwrap()
            .clone()
    }
}

impl FileWatcherLoader for BuffPool {
    fn load(resources: &mut Resources, path: &PathBuf, watcher: &mut FileWatcherSystem) {
        watcher.watch_file(path, Box::new(Self::load));
        resources.buff_pool = futures::executor::block_on(load_json(&path)).unwrap();
    }
}
