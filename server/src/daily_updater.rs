use spacetimedb::{ScheduleAt, Table};

use super::*;

#[spacetimedb::table(public, name = daily_update_timer, scheduled(daily_update_reducer))]
pub struct DailyUpdateTimer {
    #[primary_key]
    #[auto_inc]
    scheduled_id: u64,
    scheduled_at: ScheduleAt,
}

#[spacetimedb::reducer]
fn daily_update_reducer(ctx: &ReducerContext, _timer: DailyUpdateTimer) -> Result<(), String> {
    log::info!("Daily update called");
    daily_update(ctx)?;
    let next_day = (ctx
        .timestamp
        .duration_since(Timestamp::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        / 86400
        + 1)
        * 86400
        * 1000000;
    ctx.db.daily_update_timer().insert(DailyUpdateTimer {
        scheduled_id: 0,
        scheduled_at: Timestamp::from_micros_since_unix_epoch(next_day as i64).into(),
    });
    Ok(())
}

pub fn daily_update(_ctx: &ReducerContext) -> Result<(), String> {
    Ok(())
}

pub fn _daily_timer_init(ctx: &ReducerContext) {
    ctx.db.daily_update_timer().insert(DailyUpdateTimer {
        scheduled_id: 0,
        scheduled_at: ctx.timestamp.into(),
    });
}
