use itertools::Itertools;
use spacetimedb::Timestamp;

use super::*;

#[spacetimedb(table)]
pub struct GlobalData {
    #[unique]
    always_zero: u32,
    next_id: GID,
    pub game_version: String,
    pub season: u32,
    pub last_sync: Timestamp,
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
        })?;
        Ok(())
    }

    pub fn next_id() -> GID {
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
