use super::*;

#[spacetimedb(table(public))]
#[derive(Default)]
pub struct TUserStats {
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
pub struct TUserGameStats {
    #[primarykey]
    id: u64,
    season: u32,
    owner: u64,
    mode: GameMode,
    runs_played: u32,
    runs_floors: u32,
    runs_max_floor: u32,
    champion: u32,
}

impl TUserStats {
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

impl TUserGameStats {
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
        stats.runs_played += 1;
        stats.runs_floors += floor;
        stats.runs_max_floor = stats.runs_max_floor.max(floor);
        stats.save();
    }
    pub fn register_champion(owner: u64, mode: GameMode) {
        let mut stats = Self::get_or_init(owner, mode);
        stats.champion += 1;
        stats.save();
    }
}
