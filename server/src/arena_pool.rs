use rand::{seq::IteratorRandom, thread_rng};

use super::*;

#[spacetimedb(table)]
pub struct TArenaPool {
    #[primarykey]
    pub team: GID,
    pub round: u32,
}

impl TArenaPool {
    pub fn add(team: GID, round: u32) {
        TArenaPool::insert(TArenaPool { team, round }).expect("Failed to add to TArenaPool");
    }
    pub fn get_random(round: u32) -> Option<Self> {
        Self::filter_by_round(&round).choose(&mut thread_rng())
    }
}
