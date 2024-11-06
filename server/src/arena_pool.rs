use rand::seq::IteratorRandom;
use spacetimedb::Table;

use super::*;

#[spacetimedb::table(name = arena_pool)]
pub struct TArenaPool {
    #[primary_key]
    pub team: u64,
    pub mode: GameMode,
    #[index(btree)]
    pub floor: u32,
}

impl TArenaPool {
    pub fn get_next_enemy(ctx: &ReducerContext, mode: &GameMode, floor: u32) -> u64 {
        let initial_enemies = match mode {
            GameMode::ArenaNormal | GameMode::ArenaConst => GlobalData::get(ctx).initial_enemies,
            GameMode::ArenaRanked => GlobalData::get(ctx)
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
            Self::get_random(ctx, &mode, floor)
        }
    }
    pub fn add(ctx: &ReducerContext, mode: GameMode, team: u64, floor: u32) {
        ctx.db.arena_pool().insert(TArenaPool { mode, team, floor });
    }
    pub fn get_random(ctx: &ReducerContext, mode: &GameMode, floor: u32) -> u64 {
        ctx.db
            .arena_pool()
            .floor()
            .filter(floor)
            .filter(|a| a.mode.eq(mode))
            .choose(&mut ctx.rng())
            .map(|a| a.team)
            .unwrap_or_default()
    }
}
