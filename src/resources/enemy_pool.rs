use geng::prelude::itertools::Itertools;

use super::*;

#[derive(Deserialize)]
pub struct EnemyPool(Vec<PathBuf>);

impl EnemyPool {
    pub fn generate_teams(count: usize, teams: &mut Vec<PackedTeam>, resources: &Resources) {
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
        let mut names: HashSet<String> = HashSet::from_iter(teams.iter().map(|x| x.name.clone()));
        let rng = &mut thread_rng();
        while teams.len() < count {
            let unit = units.choose(rng).unwrap().clone();
            let replications = thread_rng().gen_range(2..=MAX_SLOTS);
            let mut team = PackedTeam::new(format!("{} x{replications}", unit.name), vec![unit]);
            if rng.gen::<f32>() > 0.5 {
                let buff = BuffPool::get_random(1, resources).remove(0);
                buff.apply_team_packed(&mut team);
            }
            let team: PackedTeam = ReplicatedTeam { team, replications }.into();
            if names.contains(&team.name) {
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
        println!(
            "Generated teams\n{}",
            teams
                .iter()
                .enumerate()
                .map(|(ind, team)| format!("{ind} {}", team.name))
                .join("\n")
        );
    }

    fn load() -> EnemyPool {
        futures::executor::block_on(load_json(&static_path().join("enemy_pool/_list.json")))
            .unwrap()
    }
}
