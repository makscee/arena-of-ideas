use super::*;

#[derive(Deserialize, Debug, Default)]
pub struct BuffPool {
    unit: Vec<(String, i32)>,
    team: Vec<TeamBuff>,
}

#[derive(Deserialize, Debug)]
pub struct TeamBuff {
    pub prefix: String,
    pub statuses: HashMap<String, i32>,
}

impl TeamBuff {
    pub fn apply(&self, team: &mut PackedTeam) {
        for (status, charges) in self.statuses.iter() {
            team.statuses.push((status.to_owned(), *charges));
        }
        team.name = format!("{} {}", self.prefix, team.name);
    }
}

impl BuffPool {
    pub fn random_team_buff(resources: &Resources) -> &TeamBuff {
        resources.buff_pool.team.choose(&mut thread_rng()).unwrap()
    }

    pub fn random_unit_buff(resources: &Resources) -> &(String, i32) {
        resources.buff_pool.unit.choose(&mut thread_rng()).unwrap()
    }
}

impl FileWatcherLoader for BuffPool {
    fn load(resources: &mut Resources, path: &PathBuf, watcher: &mut FileWatcherSystem) {
        watcher.watch_file(path, Box::new(Self::load));
        resources.buff_pool = futures::executor::block_on(load_json(&path)).unwrap();
    }
}
