use geng::prelude::itertools::Itertools;

use super::*;

#[derive(Deserialize)]
pub struct EnemyPool(Vec<PathBuf>);

impl EnemyPool {
    pub fn generate_teams(count: usize, resources: &Resources) -> Vec<PackedTeam> {
        let pool = Self::load();
        let units: Vec<PackedUnit> = pool
            .0
            .iter()
            .map(|path| {
                debug!("Load {path:?}");
                let unit: PackedUnit =
                    futures::executor::block_on(load_json(&static_path().join(path))).unwrap();
                unit
            })
            .collect_vec();
        let mut teams: HashMap<String, PackedTeam> = default();
        let rng = &mut thread_rng();
        while teams.len() < count {
            let unit = units.choose(rng).unwrap().clone();
            let replications = thread_rng().gen_range(1..=MAX_SLOTS);
            let mut team = PackedTeam::new(format!("{}s ({replications})", unit.name), vec![unit]);
            if rng.gen::<f32>() > 0.5 {
                BuffPool::random_team_buff(resources).apply(&mut team);
            }
            let team: PackedTeam = ReplicatedTeam { team, replications }.into();
            if teams.contains_key(&team.name) {
                continue;
            }
            teams.insert(team.name.to_owned(), team);
        }
        let teams = teams.into_values().collect_vec();
        println!(
            "Generated teams\n{}",
            teams
                .iter()
                .enumerate()
                .map(|(ind, team)| format!("{ind} {}", team.name))
                .join("\n")
        );
        teams
    }

    fn load() -> EnemyPool {
        futures::executor::block_on(load_json(&static_path().join("enemy_pool/_list.json")))
            .unwrap()
    }
}
