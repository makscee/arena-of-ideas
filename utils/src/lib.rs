use humanize_duration::prelude::DurationExt;
use std::{
    any::{type_name, type_name_of_val},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use chrono::Utc;

#[inline]
pub fn default<T: Default>() -> T {
    Default::default()
}

pub fn now_micros() -> i64 {
    Utc::now().timestamp_micros()
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
fn type_last(s: &'static str) -> &'static str {
    s.split("::").last().unwrap_or("---")
}
pub fn type_name_short<T>() -> &'static str {
    type_last(type_name::<T>())
}
pub fn type_name_of_val_short<T>(val: &T) -> &'static str {
    type_last(type_name_of_val(val))
}

pub trait Take: Sized + Default {
    fn take(&mut self) -> Self {
        std::mem::take(self)
    }
}

impl<T: Sized + Default> Take for T {}

pub trait StringUtils {
    fn cut_start(self, len: usize) -> String;
    fn cut_end(self, len: usize) -> String;
}

impl StringUtils for String {
    fn cut_start(mut self, len: usize) -> String {
        if len == 0 {
            return self;
        }
        self.drain(..self.len().max(len)).collect()
    }
    fn cut_end(mut self, len: usize) -> String {
        self.drain(self.len().saturating_sub(len)..).collect()
    }
}
