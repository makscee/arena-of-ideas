use super::*;

#[derive(Deserialize, Debug, Default)]
pub struct Ladder {
    #[serde(default)]
    current: usize,
    pub teams: Vec<ReplicatedTeam>,
}

impl Ladder {
    pub fn generate_team(&self) -> Team {
        let mut team: Team = self.teams[self.current].clone().into();
        let size = team.units.len();
        for rank in 1..=2 {
            for i in 0..size {
                let mut unit = team.units[i].clone();
                unit.rank = rank;
                team.units.push(unit);
            }
        }
        team
    }

    pub fn current_ind(&self) -> usize {
        self.current
    }

    pub fn current_replications(&self) -> usize {
        self.teams[self.current].replications
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

impl FileWatcherLoader for Ladder {
    fn loader(resources: &mut Resources, path: &PathBuf, watcher: &mut FileWatcherSystem) {
        watcher.watch_file(path, Box::new(Self::loader));
        debug!("Load floors {:?}", path);
        let prev_current = resources.ladder.current;
        resources.ladder = futures::executor::block_on(load_json(path)).unwrap();
        resources.ladder.current = prev_current.max(resources.options.start_floor);
    }
}
