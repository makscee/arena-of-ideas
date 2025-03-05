use humanize_duration::prelude::DurationExt;
use spacetimedb::Timestamp;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use chrono::Utc;

#[inline]
pub fn default<T: Default>() -> T {
    Default::default()
}

pub fn now_micros() -> i64 {
    Utc::now().timestamp_micros()
}
pub fn now_seconds() -> f64 {
    if cfg!(feature = "server") {
        Timestamp::now().to_micros_since_unix_epoch() as f64 / 1000000.0
    } else {
        Utc::now().timestamp_millis() as f64 / 1000.0
    }
}

pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let x = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}
pub fn format_timestamp(ts: u64) -> String {
    if ts == 0 {
        return "-".into();
    }
    let d = SystemTime::now().duration_since(UNIX_EPOCH).unwrap() - Duration::from_micros(ts);
    format!(
        "{} ago",
        d.human(humanize_duration::Truncate::Minute).to_string()
    )
}
pub fn format_duration(seconds: u64) -> String {
    Duration::from_secs(seconds)
        .human(humanize_duration::Truncate::Second)
        .to_string()
}
