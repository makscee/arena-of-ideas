use super::*;

#[derive(Deserialize, Debug, Default)]
pub struct Ladder {
    #[serde(default)]
    current: usize,
    pub teams: Vec<ReplicatedTeam>,
    #[serde(default)]
    tracked_units: Vec<Vec<legion::Entity>>,
}

impl Ladder {
    pub fn generate_team(resources: &Resources) -> Team {
        let ladder = &resources.ladder;
        let mut team: Team = ladder.teams[ladder.current].clone().into();
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

    pub fn track_team(world: &legion::World, resources: &mut Resources) {
        resources.ladder.tracked_units = vec![default(); 3];
        for (entity, unit) in
            UnitSystem::collect_faction_units(world, resources, Faction::Dark, true)
        {
            resources.ladder.tracked_units[unit.rank as usize].push(entity);
        }
    }

    pub fn get_score(world: &legion::World, resources: &Resources) -> usize {
        let mut score = 0;
        for entities in resources.ladder.tracked_units.iter() {
            if entities
                .iter()
                .all(|x| UnitSystem::get_corpse(*x, world).is_some())
            {
                score += 1;
            }
        }

        score
    }

    pub fn get_score_units(world: &legion::World, resources: &Resources) -> (usize, usize) {
        let units = &resources.ladder.tracked_units;
        let total = units[0].len();
        let killed = units
            .iter()
            .flatten()
            .map(|entity| match UnitSystem::get_corpse(*entity, world) {
                Some(_) => 1,
                None => 0,
            })
            .sum::<usize>();
        (killed, total)
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
