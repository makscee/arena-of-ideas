use rand::seq::IteratorRandom;

use super::*;

#[spacetimedb(table(public))]
pub struct TArenaPool {
    pub mode: GameMode,
    #[primarykey]
    pub team: u64,
    pub round: u32,
}

impl TArenaPool {
    pub fn add(mode: GameMode, team: u64, round: u32) {
        TArenaPool::insert(TArenaPool { mode, team, round }).expect("Failed to add to TArenaPool");
    }
    pub fn get_random(mode: &GameMode, round: u32) -> Option<Self> {
        Self::filter_by_round(&round)
            .filter(|d| d.mode.eq(mode))
            .choose(&mut spacetimedb::rng())
    }
}
