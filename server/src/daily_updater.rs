use super::*;

#[spacetimedb(table, scheduled(daily_update))]
struct DailyUpdateTimer {}

#[spacetimedb(reducer)]
fn daily_update(_ctx: ReducerContext, _timer: DailyUpdateTimer) -> Result<(), String> {
    self::println!("Daily update called");
    update()?;
    let next_day = (Timestamp::now()
        .duration_since(Timestamp::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        / 86400
        + 1)
        * 86400
        * 1000000;
    DailyUpdateTimer::insert(DailyUpdateTimer {
        scheduled_id: 0,
        scheduled_at: Timestamp::from_micros_since_epoch(next_day).into(),
    })
    .unwrap();
    Ok(())
}

fn update() -> Result<(), String> {
    update_constant_seed();
    TMetaShop::refresh()?;
    TDailyState::daily_refresh();
    quests_daily_refresh();
    Ok(())
}

pub fn daily_timer_init() {
    DailyUpdateTimer::insert(DailyUpdateTimer {
        scheduled_id: 0,
        scheduled_at: Timestamp::now().into(),
    })
    .unwrap();
}
