use spacetimedb::Timestamp;

use super::*;

#[spacetimedb(table)]
pub struct TArenaLeaderboard {
    pub mode: GameMode,
    pub season: u32,
    pub round: u32,
    pub score: u32,
    pub user: GID,
    pub team: GID,
    pub run: GID,
    ts: Timestamp,
}

impl TArenaLeaderboard {
    pub fn new(mode: GameMode, round: u32, score: u32, user: GID, team: GID, run: GID) -> Self {
        Self {
            mode,
            season: GlobalData::get().season,
            round,
            score,
            user,
            team,
            run,
            ts: Timestamp::now(),
        }
    }
    pub fn current_champion(mode: &GameMode) -> Option<Self> {
        TArenaLeaderboard::iter()
            .filter(|d| d.mode.eq(mode))
            .max_by_key(|d| d.round)
    }
}
