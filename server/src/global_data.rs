use spacetimedb::Table;

use super::*;

#[spacetimedb::table(public, name = global_data)]
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
    pub fn init(ctx: &ReducerContext) {
        ctx.db.global_data().insert(Self {
            always_zero: 0,
            next_id: 1,
            game_version: VERSION.to_owned(),
            last_sync: Timestamp::UNIX_EPOCH,
            initial_enemies: Vec::new(),
        });
    }

    pub fn next_id(ctx: &ReducerContext) -> u64 {
        let mut gd = Self::get(ctx);
        let id = gd.next_id;
        gd.next_id += 1;
        ctx.db.global_data().always_zero().update(gd);
        id
    }
    pub fn set_next_id(ctx: &ReducerContext, value: u64) {
        let mut gd = Self::get(ctx);
        gd.next_id = value;
        ctx.db.global_data().always_zero().update(gd);
    }

    pub fn get(ctx: &ReducerContext) -> Self {
        ctx.db.global_data().always_zero().find(0).unwrap()
    }
    pub fn register_sync(ctx: &ReducerContext) {
        let mut gd = Self::get(ctx);
        gd.last_sync = Timestamp::now();
        ctx.db.global_data().always_zero().update(gd);
    }
    pub fn set_initial_enemies(ctx: &ReducerContext, teams: Vec<u64>) {
        let mut gd = Self::get(ctx);
        gd.initial_enemies = teams;
        ctx.db.global_data().always_zero().update(gd);
    }
}
