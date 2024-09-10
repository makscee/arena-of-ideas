use spacetimedb::duration;

use super::*;

#[spacetimedb(table, scheduled(daily_update))]
struct DailyUpdateTimer {}

#[spacetimedb(reducer)]
fn daily_update(ctx: ReducerContext, arg: DailyUpdateTimer) -> Result<(), String> {
    self::println!("Daily update called");
    update_constant_seed();
    Ok(())
}

pub fn daily_timer_init() {
    DailyUpdateTimer::insert(DailyUpdateTimer {
        scheduled_id: 0,
        scheduled_at: duration!(24h).into(),
    })
    .unwrap();
}
