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
    pub fn get_next_enemy(mode: &GameMode, floor: u32) -> u64 {
        let initial_enemies = match mode {
            GameMode::ArenaNormal | GameMode::ArenaConst => GlobalData::get().initial_enemies,
            GameMode::ArenaRanked => GlobalData::get()
                .initial_enemies
                .last()
                .copied()
                .into_iter()
                .collect_vec(),
        };
        let u_floor = floor as usize;
        if u_floor < initial_enemies.len() {
            initial_enemies[u_floor]
        } else {
            Self::get_random(&mode, floor)
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
