use spacetimedb::Table;

use super::*;

#[spacetimedb::table(public, name = global_data)]
pub struct GlobalData {
    #[unique]
    pub always_zero: u32,
    next_id: u64,
    pub game_version: String,
    pub last_sync: Timestamp,
}

const VERSION: &str = env!("CARGO_PKG_VERSION");
impl GlobalData {
    pub fn init(ctx: &ReducerContext) {
        ctx.db.global_data().always_zero().delete(0);
        ctx.db.global_data().insert(Self {
            always_zero: 0,
            next_id: 0,
            game_version: VERSION.to_owned(),
            last_sync: Timestamp::UNIX_EPOCH,
        });
    }

    pub fn next_id(ctx: &ReducerContext) -> u64 {
        let mut gd = Self::get(ctx);
        let ts = ctx.timestamp.to_micros_since_unix_epoch() as u64;
        gd.next_id = ts.max(gd.next_id + 1);
        let id = gd.next_id;
        ctx.db.global_data().always_zero().update(gd);
        id
    }

    pub fn get(ctx: &ReducerContext) -> Self {
        ctx.db.global_data().always_zero().find(0).unwrap()
    }
    pub fn register_sync(ctx: &ReducerContext) {
        let mut gd = Self::get(ctx);
        gd.last_sync = Timestamp::now();
        ctx.db.global_data().always_zero().update(gd);
    }
}
