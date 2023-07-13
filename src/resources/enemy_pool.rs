use geng::prelude::itertools::Itertools;

use super::*;

#[derive(Deserialize, Default)]
pub struct EnemyPool(HashMap<String, PackedUnit>);

impl EnemyPool {
    pub fn get_unit_by_name(name: &str, resources: &Resources) -> PackedUnit {
        resources.enemy_pool.0.get(name).cloned().unwrap()
    }

    pub fn get_random_unit(resources: &Resources) -> PackedUnit {
        resources
            .enemy_pool
            .0
            .values()
            .choose(&mut thread_rng())
            .unwrap()
            .clone()
    }

    pub fn fill_teams_vec(count: usize, teams: &mut Vec<PackedTeam>, resources: &Resources) {
        if count == teams.len() {
            return;
        }
        let mut names: HashSet<String> = HashSet::from_iter(teams.iter().map(|x| x.name.clone()));
        let rng = &mut thread_rng();
        while teams.len() < count {
            let unit = Self::get_random_unit(resources);
            let replications = thread_rng().gen_range(2..=MAX_SLOTS);
            let mut team = PackedTeam::new(format!("{} x{replications}", unit.name), vec![unit]);
            if rng.gen::<f32>() > 0.75 {
                let buff = BuffPool::get_random(1, resources).remove(0);
                buff.apply_team_packed(&mut team);
            }
            let team: PackedTeam = ReplicatedTeam { team, replications }.into();
            let name = team.name.split(" x").collect_vec()[0];
            if names.contains(&team.name)
                || names.contains(&format!("{name} x{}", replications + 1))
                || names.contains(&format!("{name} x{}", replications - 1))
            {
                continue;
            }
            names.insert(team.name.clone());
            teams.push(team);
        }
        teams.sort_by_key(|x| {
            let mut s = x.name.split(' ').map(|x| x.to_string()).collect_vec();
            if s.len() == 2 {
                s.insert(0, String::new());
            }
            (s[1].clone(), s[0].clone(), s[2].clone())
        });
    }
}

impl FileWatcherLoader for EnemyPool {
    fn load(resources: &mut Resources, path: &PathBuf, watcher: &mut FileWatcherSystem) {
        watcher.watch_file(path, Box::new(Self::load));
        debug!("Load enemy pool {path:?}");
        let paths: Vec<PathBuf> = futures::executor::block_on(load_json(path)).unwrap();
        for path in paths.iter() {
            let unit: PackedUnit =
                futures::executor::block_on(load_json(&static_path().join(path))).unwrap();
            resources.enemy_pool.0.insert(unit.name.clone(), unit);
        }
        debug!("Loaded enemies:\n{:?}", paths);
    }
}
