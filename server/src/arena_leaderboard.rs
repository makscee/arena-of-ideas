use spacetimedb::Timestamp;

use super::*;

#[spacetimedb(table(public))]
pub struct TArenaLeaderboard {
    pub mode: GameMode,
    pub season: u32,
    pub floor: u32,
    pub user: u64,
    pub team: u64,
    pub run: u64,
    ts: Timestamp,
}

impl TArenaLeaderboard {
    pub fn new(mode: GameMode, floor: u32, user: u64, team: u64, run: u64) -> Self {
        Self {
            mode,
            season: GlobalData::get().season,
            floor,
            user,
            team,
            run,
            ts: Timestamp::now(),
        }
    }
    pub fn current_champion(mode: &GameMode) -> Option<Self> {
        TArenaLeaderboard::iter()
            .filter(|d| d.mode.eq(mode))
            .max_by_key(|d| d.floor)
    }
}
