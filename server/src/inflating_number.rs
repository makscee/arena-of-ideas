use super::*;

#[derive(SpacetimeType)]
pub struct InflatingInt {
    pub start: i64,
    pub max: i64,
    pub inflation: f64,
}

impl InflatingInt {
    pub fn value(&self, count: i64) -> i64 {
        (self.start + (self.inflation * count as f64) as i64).min(self.max)
    }
}
