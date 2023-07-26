use super::*;

#[derive(Deserialize, Clone)]
pub struct Curse {
    pub positive_effect: EffectWrapped,
    pub positive_text: String,
    pub negative_effect: EffectWrapped,
    pub negative_text: String,
}

#[derive(Deserialize, Default)]
pub struct CursePool {
    pub curses: Vec<Curse>,
}

impl CursePool {
    pub fn get_random(resources: &Resources) -> Curse {
        resources
            .curse_pool
            .curses
            .choose(&mut thread_rng())
            .unwrap()
            .clone()
    }
}

impl FileWatcherLoader for CursePool {
    fn load(resources: &mut Resources, path: &PathBuf, watcher: &mut FileWatcherSystem) {
        watcher.watch_file(path, Box::new(Self::load));
        resources.curse_pool = futures::executor::block_on(load_json(&path)).unwrap();
    }
}
