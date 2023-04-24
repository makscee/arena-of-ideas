use geng::prelude::itertools::Itertools;

use super::*;

#[derive(Deserialize)]
pub struct EnemyPool {
    pub teams: Vec<PathBuf>,
    pub buffs: Vec<TeamBuff>,
}

#[derive(Deserialize)]
pub struct TeamBuff {
    pub prefix: String,
    pub statuses: HashMap<String, i32>,
}

impl EnemyPool {
    pub fn generate_teams() -> Vec<PackedTeam> {
        let pool = Self::load();
        let mut teams: Vec<PackedTeam> = pool
            .teams
            .iter()
            .map(|path| {
                debug!("Load {path:?}");
                let team: ReplicatedTeam =
                    futures::executor::block_on(load_json(&static_path().join(path))).unwrap();
                let team: PackedTeam = team.into();
                team
            })
            .collect_vec();
        for _ in 0..25 {
            let mut team = teams.choose(&mut thread_rng()).unwrap().clone();
            let buff = pool.buffs.choose(&mut thread_rng()).unwrap();
            team.statuses.extend(buff.statuses.clone().into_iter());
            team.name = format!("{} {}", buff.prefix.clone(), team.name);
            teams.push(team);
        }
        teams
    }

    fn load() -> EnemyPool {
        futures::executor::block_on(load_json(&static_path().join("enemy_pool/_config.json")))
            .unwrap()
    }
}
