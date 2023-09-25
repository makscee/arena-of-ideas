use rand::seq::{IteratorRandom, SliceRandom};

use super::*;

pub struct RatingPlugin;

impl RatingPlugin {
    pub fn generate_weakest_opponent(team: &PackedTeam, world: &mut World) -> PackedTeam {
        let mut enemies = Pools::get(world)
            .enemies
            .values()
            .choose_multiple(&mut thread_rng(), 2)
            .into_iter()
            .map(|x| PackedTeam::new(vec![x.clone()]))
            .collect_vec();
        let mut candidates = Vec::default();
        while candidates.len() < 2 {
            let ind = (&mut thread_rng()).gen_range(0..enemies.len());
            let enemy = &mut enemies[ind];
            match SimulationPlugin::run(team.clone(), enemy.clone(), world) {
                BattleResult::Right(_) => candidates.push(enemies.remove(ind)),
                _ => Self::strenghten(enemy),
            }
        }
        mem::take(candidates.choose_mut(&mut thread_rng()).unwrap())
    }

    fn strenghten(team: &mut PackedTeam) {
        team.units.push(team.units[0].clone())
    }
}
