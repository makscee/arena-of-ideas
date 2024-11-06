use spacetimedb::Timestamp;

use super::*;

#[spacetimedb::table(name = arena_leaderboard)]
pub struct TArenaLeaderboard {
    pub mode: GameMode,
    #[index(btree)]
    pub season: u32,
    pub floor: u32,
    pub owner: u64,
    pub team: u64,
    pub run: u64,
    ts: Timestamp,
}

impl TArenaLeaderboard {
    pub fn new(
        ctx: &ReducerContext,
        mode: GameMode,
        floor: u32,
        owner: u64,
        team: u64,
        run: u64,
    ) -> Self {
        Self {
            mode,
            season: GlobalSettings::get(ctx).season,
            floor,
            owner,
            team,
            run,
            ts: Timestamp::now(),
        }
    }
    pub fn current(ctx: &ReducerContext) -> impl Iterator<Item = Self> {
        ctx.db
            .arena_leaderboard()
            .season()
            .filter(GlobalSettings::get(ctx).season)
    }
    pub fn floor_boss(ctx: &ReducerContext, mode: GameMode, floor: u32) -> Option<Self> {
        Self::current(ctx)
            .filter(|d| d.floor == floor && d.mode == mode)
            .max_by_key(|d| d.ts)
    }
    pub fn current_champion(ctx: &ReducerContext, mode: GameMode) -> Option<Self> {
        ctx.db
            .arena_leaderboard()
            .season()
            .filter(GlobalSettings::get(ctx).season)
            .filter(|d| d.mode.eq(&mode))
            .max_by_key(|d| d.floor)
    }
}
