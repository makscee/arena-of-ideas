use super::*;

#[derive(Deserialize, Debug, Clone, Default)]
pub struct Floors {
    #[serde(default)]
    current: usize,
    floors: Vec<Team>,
}

impl Floors {
    pub fn current(&self) -> &Team {
        &self.floors[self.current]
    }

    pub fn current_ind(&self) -> usize {
        self.current
    }

    pub fn reset(&mut self) {
        self.current = default();
    }

    pub fn next(&mut self) -> bool {
        self.current += 1;
        self.current < self.floors.len()
    }
}

impl FileWatcherLoader for Floors {
    fn loader(resources: &mut Resources, path: &PathBuf, watcher: &mut FileWatcherSystem) {
        watcher.watch_file(path, Box::new(Self::loader));
        resources.floors = futures::executor::block_on(load_json(path)).unwrap();
    }
}
