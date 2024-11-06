use super::*;

#[spacetimedb::table(name = player_stats)]
#[derive(Default)]
pub struct TPlayerStats {
    #[primary_key]
    id: u64,
    #[index(btree)]
    season: u32,
    #[index(btree)]
    owner: u64,
    time_played: u64,
    quests_completed: u32,
    credits_earned: u32,
}

#[spacetimedb::table(name = player_game_stats)]
#[derive(Default)]
pub struct TPlayerGameStats {
    #[primary_key]
    id: u64,
    #[index(btree)]
    season: u32,
    #[index(btree)]
    owner: u64,
    mode: GameMode,
    runs: u32,
    floors: Vec<u32>,
    champion: u32,
    boss: u32,
}

impl TPlayerStats {
    fn get_or_init(ctx: &ReducerContext, owner: u64) -> Self {
        let season = GlobalSettings::get(ctx).season;
        ctx.db
            .player_stats()
            .owner()
            .filter(owner)
            .filter(|d| d.season == season)
            .next()
            .unwrap_or_else(|| {
                ctx.db.player_stats().insert(Self {
                    owner,
                    season,
                    ..default()
                })
            })
    }
    fn save(self, ctx: &ReducerContext) {
        ctx.db.player_stats().id().update(self);
    }
    pub fn register_time_played(ctx: &ReducerContext, owner: u64, time: u64) {
        let mut stats = Self::get_or_init(ctx, owner);
        stats.time_played += time;
        stats.save(ctx);
    }
    pub fn register_credits_earned(ctx: &ReducerContext, owner: u64, value: u32) {
        let mut stats = Self::get_or_init(ctx, owner);
        stats.credits_earned += value;
        stats.save(ctx);
    }
    pub fn register_completed_quest(ctx: &ReducerContext, owner: u64) {
        let mut stats = Self::get_or_init(ctx, owner);
        stats.quests_completed += 1;
        stats.save(ctx);
    }
}

impl TPlayerGameStats {
    fn get_or_init(ctx: &ReducerContext, owner: u64, mode: GameMode) -> Self {
        let season = GlobalSettings::get(ctx).season;
        ctx.db
            .player_game_stats()
            .owner()
            .filter(owner)
            .filter(|d| d.mode == mode && d.season == season)
            .next()
            .unwrap_or_else(|| {
                ctx.db.player_game_stats().insert(Self {
                    id: next_id(ctx),
                    owner,
                    season,
                    ..default()
                })
            })
    }
    fn save(self, ctx: &ReducerContext) {
        ctx.db.player_game_stats().id().update(self);
    }
    pub fn register_run_end(ctx: &ReducerContext, owner: u64, mode: GameMode, floor: u32) {
        let mut stats = Self::get_or_init(ctx, owner, mode);
        stats.runs += 1;
        let u = floor as usize;
        if stats.floors.len() < u + 1 {
            stats.floors.resize(u + 1, 0);
        }
        stats.floors[u] += 1;
        stats.save(ctx);
    }
    pub fn register_champion(ctx: &ReducerContext, owner: u64, mode: GameMode) {
        let mut stats = Self::get_or_init(ctx, owner, mode);
        stats.champion += 1;
        stats.save(ctx);
    }
    pub fn register_boss(ctx: &ReducerContext, owner: u64, mode: GameMode) {
        let mut stats = Self::get_or_init(ctx, owner, mode);
        stats.boss += 1;
        stats.save(ctx);
    }
}
