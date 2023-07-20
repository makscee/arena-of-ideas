use geng::prelude::itertools::Itertools;

use super::*;

#[derive(Debug, Default)]
pub struct Ladder {
    current: usize,
    pub levels: Vec<ReplicatedTeam>,
}

impl Ladder {
    pub fn start_next_battle(world: &mut legion::World, resources: &mut Resources) {
        let light = PackedTeam::pack(Faction::Team, world, resources);
        let dark = resources
            .ladder
            .levels
            .get(Self::current_level(resources))
            .unwrap()
            .clone();
        BattleSystem::init_battle(&light, &dark.into(), world, resources);
        GameStateSystem::set_transition(GameState::Battle, resources);
    }

    pub fn get_score(world: &legion::World) -> usize {
        (UnitSystem::collect_faction(world, Faction::Dark).len() == 0) as usize
    }

    // pub fn set_teams(teams: Vec<PackedTeam>, resources: &mut Resources) {
    //     resources.ladder.levels = teams.into_iter().map(|x| x.into()).collect_vec();
    // }

    // pub fn save(resources: &Resources) {
    //     let path = static_path().join("ladder.json");
    //     let data = serde_json::to_string_pretty(&resources.ladder.levels).unwrap();
    //     match std::fs::write(&path, data) {
    //         Ok(_) => debug!("Save ladder to {:?}", &path),
    //         Err(error) => error!("Can't save ladder: {}", error),
    //     }
    // }

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
