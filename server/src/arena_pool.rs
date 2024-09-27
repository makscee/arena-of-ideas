use rand::seq::IteratorRandom;

use super::*;

#[spacetimedb(table(public))]
pub struct TArenaPool {
    #[primarykey]
    pub team: u64,
    pub mode: GameMode,
    pub floor: u32,
}

impl TArenaPool {
    pub fn get_next_enemy(mode: &GameMode, floor: u32) -> (bool, u64) {
        let initial_enemies = match mode {
            GameMode::ArenaNormal | GameMode::ArenaConst(_) => GlobalData::get().initial_enemies,
            GameMode::ArenaRanked => GlobalData::get()
                .initial_enemies
                .last()
                .copied()
                .into_iter()
                .collect_vec(),
        };
        let u_floor = floor as usize;
        let champion = TArenaLeaderboard::current_champion(&mode);
        if u_floor < initial_enemies.len() {
            let enemy = initial_enemies[u_floor];
            return (
                u_floor == initial_enemies.len() - 1 && champion.is_none(),
                enemy,
            );
        } else if champion.as_ref().is_some_and(|c| c.floor == floor) {
            return (true, champion.unwrap().team);
        } else {
            return (false, Self::get_random(&mode, floor));
        }
    }
    pub fn add(mode: GameMode, team: u64, floor: u32) {
        TArenaPool::insert(TArenaPool { mode, team, floor }).expect("Failed to add to TArenaPool");
    }
    pub fn get_random(mode: &GameMode, floor: u32) -> u64 {
        Self::filter_by_floor(&floor)
            .filter(|d| d.mode.eq(mode))
            .choose(&mut spacetimedb::rng())
            .map(|p| p.team)
            .unwrap_or_default()
    }
}
