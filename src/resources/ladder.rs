use geng::prelude::itertools::Itertools;

use super::*;

#[derive(Debug, Default)]
pub struct Ladder {
    current: usize,
    pub levels: Vec<LadderLevel>,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct LadderLevel {
    pub teams: Vec<LadderTeam>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct LadderTeam {
    pub enemy_name: String,
    pub count: usize,
    #[serde(default)]
    pub buff: Option<String>,
}

impl LadderLevel {
    pub fn generate_teams(&self, resources: &Resources) -> Vec<PackedTeam> {
        self.teams
            .iter()
            .map(|x| x.generate_team(resources))
            .collect_vec()
    }
}

impl LadderTeam {
    pub fn generate_team(&self, resources: &Resources) -> PackedTeam {
        let mut team: PackedTeam = ReplicatedTeam {
            team: PackedTeam::from_units(vec![EnemyPool::get_unit_by_name(
                &self.enemy_name,
                resources,
            )]),
            replications: self.count,
        }
        .into();
        team.name = format!("{} x{}", team.name, self.count);
        if let Some(buff) = self.buff.as_ref() {
            BuffPool::get_by_name(buff, resources).apply_team_packed(&mut team);
        }
        team
    }

    pub fn from_packed_team(team: &PackedTeam) -> Self {
        let buff = match team.statuses.is_empty() {
            false => Some(team.statuses[0].0.clone()),
            true => None,
        };
        Self {
            enemy_name: team.units[0].name.clone(),
            count: team.units.len(),
            buff,
        }
    }
}

impl Ladder {
    pub fn generate_promoted_teams(team: PackedTeam) -> Vec<PackedTeam> {
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

    pub fn set_teams(teams: &Vec<PackedTeam>, resources: &mut Resources) {
        let mut level = LadderLevel::default();
        resources.ladder.levels = default();
        for team in teams {
            level.teams.push(LadderTeam::from_packed_team(team));
            if level.teams.len() == 3 {
                resources.ladder.levels.push(level);
                level = default();
            }
        }
    }

    pub fn save(resources: &Resources) {
        let path = static_path().join("ladder.json");
        let data = serde_json::to_string_pretty(&resources.ladder.levels).unwrap();
        match std::fs::write(&path, data) {
            Ok(_) => debug!("Save ladder to {:?}", &path),
            Err(error) => error!("Can't save ladder: {}", error),
        }
    }

    pub fn generate_current_teams(resources: &Resources) -> Vec<PackedTeam> {
        resources.ladder.levels[resources.ladder.current].generate_teams(resources)
    }

    pub fn current_level(resources: &Resources) -> usize {
        resources.ladder.current
    }

    pub fn reset(resources: &mut Resources) {
        resources.ladder.current = default();
    }

    pub fn next(resources: &mut Resources) -> bool {
        resources.ladder.current += 1;
        resources.ladder.current < resources.ladder.count()
    }

    pub fn count(&self) -> usize {
        self.levels.len()
    }

    pub fn set_level(&mut self, ind: usize) {
        self.current = ind;
    }

    pub fn all_teams(resources: &Resources) -> Vec<PackedTeam> {
        resources
            .ladder
            .levels
            .iter()
            .map(|x| {
                x.teams
                    .iter()
                    .map(|x| x.generate_team(resources))
                    .collect_vec()
            })
            .flatten()
            .collect_vec()
    }
}

impl FileWatcherLoader for Ladder {
    fn load(resources: &mut Resources, path: &PathBuf, watcher: &mut FileWatcherSystem) {
        watcher.watch_file(path, Box::new(Self::load));
        debug!("Load ladder {path:?}");
        let prev_current = resources.ladder.current;
        resources.ladder.levels = futures::executor::block_on(load_json(path)).unwrap();
        resources.ladder.current = prev_current;
        debug!(
            "Loaded {} levels, current level {prev_current}",
            resources.ladder.levels.len()
        );
    }
}
