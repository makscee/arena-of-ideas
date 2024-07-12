use super::*;

#[spacetimedb(table)]
pub struct TArenaLeaderboard {
    pub season: u32,
    pub round: u32,
    pub score: u32,
    pub user: GID,
    pub team: GID,
    pub run: GID,
}

impl TArenaLeaderboard {
    pub fn new(round: u32, score: u32, user: GID, team: GID, run: GID) -> Self {
        Self {
            season: GlobalData::get().season,
            round,
            score,
            user,
            team,
            run,
        }
    }
}
