use itertools::Itertools;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use spacetimedb::{println, schedule, Timestamp};

use super::*;

#[spacetimedb(table)]
pub struct GlobalData {
    #[unique]
    pub always_zero: u32,
    next_id: u64,
    pub game_version: String,
    pub season: u32,
    pub last_sync: Timestamp,
    pub constant_seed: String,
}

const VERSION: &str = env!("CARGO_PKG_VERSION");
impl GlobalData {
    pub fn init() -> Result<(), String> {
        let season = VERSION.split(".").collect_vec()[1].parse().unwrap();
        GlobalData::insert(GlobalData {
            always_zero: 0,
            next_id: 1,
            game_version: VERSION.to_owned(),
            last_sync: Timestamp::UNIX_EPOCH,
            season,
            constant_seed: String::new(),
        })?;
        schedule!("1ms", update_constant_seed());
        Ok(())
    }

    pub fn next_id() -> u64 {
        let mut gd = GlobalData::filter_by_always_zero(&0).unwrap();
        let id = gd.next_id;
        gd.next_id += 1;
        GlobalData::update_by_always_zero(&0, gd);
        id
    }

    pub fn get() -> Self {
        GlobalData::filter_by_always_zero(&0).unwrap()
    }
    pub fn register_sync() {
        let mut gd = Self::get();
        gd.last_sync = Timestamp::now();
        Self::update_by_always_zero(&0, gd);
    }
}

fn generate_str_seed(count: usize) -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(count)
        .map(char::from)
        .collect()
}

#[spacetimedb(reducer)]
fn update_constant_seed() {
    let mut gd = GlobalData::get();
    if gd.constant_seed.is_empty()
        || TArenaLeaderboard::current_champion(&GameMode::ArenaConst(gd.constant_seed.into()))
            .is_some_and(|d| d.round >= 10)
    {
        let seed = generate_str_seed(10);
        println!("Constant seed updated to {seed}");
        gd.constant_seed = seed;
        GlobalData::update_by_always_zero(&0, gd);
    }
    schedule!("3h", update_constant_seed());
}
