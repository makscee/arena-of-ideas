use super::*;

#[spacetimedb(table(public))]
pub struct GlobalData {
    #[unique]
    pub always_zero: u32,
    next_id: u64,
    pub game_version: String,
    pub last_sync: Timestamp,
    pub initial_enemies: Vec<u64>,
}

const VERSION: &str = env!("CARGO_PKG_VERSION");
impl GlobalData {
    pub fn init() -> Result<(), String> {
        GlobalData::insert(GlobalData {
            always_zero: 0,
            next_id: 1,
            game_version: VERSION.to_owned(),
            last_sync: Timestamp::UNIX_EPOCH,
            initial_enemies: Vec::new(),
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
    pub fn set_next_id(value: u64) {
        let mut gd = GlobalData::filter_by_always_zero(&0).unwrap();
        gd.next_id = value;
        GlobalData::update_by_always_zero(&0, gd);
    }

    pub fn get() -> Self {
        GlobalData::filter_by_always_zero(&0).unwrap()
    }
    pub fn register_sync() {
        let mut gd = Self::get();
        gd.last_sync = Timestamp::now();
        Self::update_by_always_zero(&0, gd);
    }
    pub fn set_initial_enemies(teams: Vec<u64>) {
        let mut gd = Self::get();
        gd.initial_enemies = teams;
        Self::update_by_always_zero(&0, gd);
    }
}
