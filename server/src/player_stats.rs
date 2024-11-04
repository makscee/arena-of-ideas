use super::*;

#[spacetimedb(table(public))]
#[derive(Default)]
pub struct TPlayerStats {
    #[primarykey]
    id: u64,
    season: u32,
    owner: u64,
    time_played: u64,
    quests_completed: u32,
    credits_earned: u32,
}

#[spacetimedb(table(public))]
#[derive(Default)]
pub struct TPlayerGameStats {
    #[primarykey]
    id: u64,
    season: u32,
    owner: u64,
    mode: GameMode,
    runs: u32,
    floors: Vec<u32>,
    champion: u32,
    boss: u32,
}

impl TPlayerStats {
    fn get_or_init(owner: u64) -> Self {
        let season = GlobalSettings::get().season;
        Self::filter_by_owner(&owner)
            .filter(|d| d.season == season)
            .next()
            .unwrap_or_else(|| {
                Self::insert(Self {
                    owner,
                    season,
                    ..default()
                })
                .unwrap()
            })
    }
    fn save(self) {
        Self::update_by_id(&self.id.clone(), self);
    }
    pub fn register_time_played(owner: u64, time: u64) {
        let mut stats = Self::get_or_init(owner);
        stats.time_played += time;
        stats.save();
    }
    pub fn register_credits_earned(owner: u64, value: u32) {
        let mut stats = Self::get_or_init(owner);
        stats.credits_earned += value;
        stats.save();
    }
    pub fn register_completed_quest(owner: u64) {
        let mut stats = Self::get_or_init(owner);
        stats.quests_completed += 1;
        stats.save();
    }
}

impl TPlayerGameStats {
    fn get_or_init(owner: u64, mode: GameMode) -> Self {
        let season = GlobalSettings::get().season;
        Self::filter_by_owner(&owner)
            .filter(|d| d.mode == mode && d.season == season)
            .next()
            .unwrap_or_else(|| {
                Self::insert(Self {
                    id: next_id(),
                    owner,
                    season,
                    ..default()
                })
                .unwrap()
            })
    }
    fn save(self) {
        Self::update_by_id(&self.id.clone(), self);
    }
    pub fn register_run_end(owner: u64, mode: GameMode, floor: u32) {
        let mut stats = Self::get_or_init(owner, mode);
        stats.runs += 1;
        let u = floor as usize;
        if stats.floors.len() < u + 1 {
            stats.floors.resize(u + 1, 0);
        }
        stats.floors[u] += 1;
        stats.save();
    }
    pub fn register_champion(owner: u64, mode: GameMode) {
        let mut stats = Self::get_or_init(owner, mode);
        stats.champion += 1;
        stats.save();
    }
    pub fn register_boss(owner: u64, mode: GameMode) {
        let mut stats = Self::get_or_init(owner, mode);
        stats.boss += 1;
        stats.save();
    }
}
