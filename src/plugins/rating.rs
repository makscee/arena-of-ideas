use rand::seq::IteratorRandom;

use super::*;

pub struct RatingPlugin;

impl RatingPlugin {
    pub fn generate_weakest_opponent(
        team: &PackedTeam,
        count: usize,
        world: &mut World,
    ) -> Vec<PackedTeam> {
        let enemies = Pools::get(world).enemies.values().collect_vec();
        let mut enemies = (0..count * 2)
            .map(|_| {
                PackedTeam::new(vec![enemies
                    .iter()
                    .choose(&mut thread_rng())
                    .unwrap()
                    .to_owned()
                    .clone()])
            })
            .collect_vec();
        let mut candidates = Vec::default();
        while candidates.len() < count {
            let ind = (&mut thread_rng()).gen_range(0..enemies.len());
            let enemy = &mut enemies[ind];
            match SimulationPlugin::run(team.clone(), enemy.clone(), world) {
                Ok(result) => match result {
                    BattleResult::Right(..) => candidates.push(enemies.remove(ind)),
                    BattleResult::Left(count) => {
                        for _ in 0..(0..=count).into_iter().choose(&mut thread_rng()).unwrap() {
                            Self::strenghten(enemy, world);
                        }
                    }
                    _ => Self::strenghten(enemy, world),
                },
                Err(err) => panic!("Battle Run error: {err}"),
            }
            SimulationPlugin::clear(world);
        }
        candidates
    }

    fn strenghten(team: &mut PackedTeam, world: &mut World) {
        let rng = &mut thread_rng();
        if rng.gen_ratio(1, team.units.len() as u32) {
            team.units.push(team.units[0].clone())
        } else {
            let status = if let Some((status, _)) = team.units[0].statuses.first() {
                status.to_owned()
            } else {
                Pools::get(world)
                    .statuses
                    .iter()
                    .choose(&mut thread_rng())
                    .unwrap()
                    .0
                    .to_owned()
            };
            PackedStatus::apply_to_team(&status, 1, team);
        }
    }
}
