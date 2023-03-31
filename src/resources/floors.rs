use super::*;

#[derive(Deserialize, Debug, Clone, Default)]
pub struct Floors {
    #[serde(default)]
    current: usize,
    pub teams: Vec<Team>,
}

impl Floors {
    pub fn current(&self) -> &Team {
        &self.teams[self.current]
    }

    pub fn current_ind(&self) -> usize {
        self.current
    }

    pub fn reset(&mut self) {
        self.current = default();
    }

    pub fn next(&mut self) -> bool {
        self.current += 1;
        self.current < self.teams.len()
    }

    pub fn count(&self) -> usize {
        self.teams.len()
    }

    pub fn set(&mut self, ind: usize) {
        self.current = ind;
    }
}

impl FileWatcherLoader for Floors {
    fn loader(resources: &mut Resources, path: &PathBuf, watcher: &mut FileWatcherSystem) {
        watcher.watch_file(path, Box::new(Self::loader));
        debug!("Load floors {:?}", path);
        resources.floors = futures::executor::block_on(load_json(path)).unwrap();
    }
}
