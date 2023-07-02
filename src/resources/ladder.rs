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
        ladder.teams[ladder.current].clone().into()
    }

    pub fn get_current_teams(resources: &Resources) -> Vec<&PackedTeam> {
        let ladder = &resources.ladder;
        let current = ladder.current;
        vec![
            &ladder.teams[current],
            &ladder.teams[current + 1],
            &ladder.teams[current + 2],
        ]
    }

    pub fn generate_teams(team: PackedTeam) -> Vec<PackedTeam> {
        let mut r1 = team.clone();
        for unit in r1.units.iter_mut() {
            unit.rank = 1;
        }
        let mut r2 = team.clone();
        for unit in r2.units.iter_mut() {
            unit.rank = 2;
        }
        vec![team, r1, r2]
    }

    pub fn get_score(world: &legion::World) -> usize {
        let mut min_rank = 3;
        for (_, state) in UnitSystem::collect_faction_states(world, Faction::Dark) {
            min_rank = min_rank.min(state.try_get_int(&VarName::Rank, world).unwrap_or_default());
        }
        min_rank as usize
    }

    pub fn current_ind(&self) -> usize {
        self.current
    }

    pub fn current_level(&self) -> usize {
        self.current / 3
    }

    pub fn reset(&mut self) {
        self.current = default();
    }

    pub fn next(&mut self) -> bool {
        self.current += 3;
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
        debug!(
            "Loaded {} teams, current level {prev_current}",
            resources.ladder.teams.len()
        );
    }
}
