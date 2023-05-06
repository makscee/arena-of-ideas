use super::*;

#[derive(Deserialize, Debug, Default)]
pub struct Ladder {
    #[serde(default)]
    current: usize,
    pub teams: Vec<PackedTeam>,
}

impl Ladder {
    pub fn load_team(resources: &Resources) -> PackedTeam {
        let ladder = &resources.ladder;
        Self::generate_team(ladder.teams[ladder.current].clone().into())
    }

    pub fn generate_team(mut team: PackedTeam) -> PackedTeam {
        return team;
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

    pub fn get_score(world: &legion::World, resources: &Resources) -> usize {
        match UnitSystem::collect_faction(world, Faction::Dark).len() > 0 {
            true => 0,
            false => 1,
        }
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

impl FileWatcherLoader for Ladder {
    fn load(resources: &mut Resources, path: &PathBuf, watcher: &mut FileWatcherSystem) {
        watcher.watch_file(path, Box::new(Self::load));
        debug!("Load ladder {path:?}");
        let prev_current = resources.ladder.current;
        resources.ladder.teams = futures::executor::block_on(load_json(path)).unwrap();
        resources.ladder.current = prev_current;
    }
}
