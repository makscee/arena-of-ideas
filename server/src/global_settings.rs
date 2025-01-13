use spacetimedb::Table;

use super::*;

#[spacetimedb::table(public, name = global_settings)]
pub struct GlobalSettings {
    #[unique]
    always_zero: u32,
    pub hero_speed: f32,
}

impl GlobalSettings {
    pub fn get(ctx: &ReducerContext) -> Self {
        ctx.db.global_settings().always_zero().find(0).unwrap()
    }
    pub fn replace(self, ctx: &ReducerContext) {
        ctx.db.global_settings().always_zero().delete(0);
        ctx.db.global_settings().insert(self);
    }
}
