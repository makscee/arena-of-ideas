use spacetimedb::Timestamp;

use super::*;

#[spacetimedb(table)]
pub struct GlobalData {
    #[unique]
    always_zero: u32,
    next_id: u64,
    pub game_version: String,
    pub last_sync: Timestamp,
}

const VERSION: &str = env!("CARGO_PKG_VERSION");
impl GlobalData {
    pub fn init() -> Result<(), String> {
        GlobalData::insert(GlobalData {
            always_zero: 0,
            next_id: 1,
            game_version: VERSION.to_owned(),
            last_sync: Timestamp::UNIX_EPOCH,
        })?;
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
}
