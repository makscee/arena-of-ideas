use spacetimedb::Table;

use super::*;

#[spacetimedb::table(public, name = daily_update_timer, scheduled(daily_update))]
pub struct DailyUpdateTimer {}

#[spacetimedb::reducer]
fn daily_update(ctx: &ReducerContext, _timer: DailyUpdateTimer) -> Result<(), String> {
    self::println!("Daily update called");
    update(ctx)?;
    let next_day = (Timestamp::now()
        .duration_since(Timestamp::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        / 86400
        + 1)
        * 86400
        * 1000000;
    ctx.db.daily_update_timer().insert(DailyUpdateTimer {
        scheduled_id: 0,
        scheduled_at: Timestamp::from_micros_since_epoch(next_day).into(),
    });
    Ok(())
}

fn update(ctx: &ReducerContext) -> Result<(), String> {
    TMetaShop::refresh(ctx)?;
    TDailyState::daily_refresh(ctx);
    quests_daily_refresh(ctx);
    TPlayer::cleanup(ctx);
    Ok(())
}

pub fn daily_timer_init(ctx: &ReducerContext) {
    ctx.db.daily_update_timer().insert(DailyUpdateTimer {
        scheduled_id: 0,
        scheduled_at: Timestamp::now().into(),
    });
}
