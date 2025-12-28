use crate::*;
use serde::{Deserialize, Serialize};
use std::convert::TryInto;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct History<T: Clone + PartialEq> {
    entries: Vec<(f32, T)>,
    current: T,
}

impl<T: Clone + PartialEq + Default> Default for History<T> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            current: T::default(),
        }
    }
}

impl<T: Clone + PartialEq> History<T> {
    pub fn new(initial: T) -> Self {
        Self {
            entries: Vec::new(),
            current: initial,
        }
    }

    pub fn insert(&mut self, t: f32, value: T) {
        if self.current != value {
            self.current = value.clone();
            // Insert maintaining sorted order by time
            match self
                .entries
                .binary_search_by(|(hist_t, _)| hist_t.total_cmp(&t))
            {
                Ok(pos) => {
                    // Replace existing entry at same time
                    self.entries[pos] = (t, value);
                }
                Err(pos) => {
                    // Insert at correct position
                    self.entries.insert(pos, (t, value));
                }
            }
        }
    }

    pub fn value_at(&self, t: f32) -> Option<T> {
        if self.entries.is_empty() || t < 0.0 {
            return None;
        }

        // Binary search for the right time point
        match self
            .entries
            .binary_search_by(|(hist_t, _)| hist_t.total_cmp(&t))
        {
            Ok(pos) => {
                // Exact match at time t
                Some(self.entries[pos].1.clone())
            }
            Err(pos) => {
                if pos == 0 {
                    // t is before first entry
                    None
                } else {
                    // Return value from previous entry
                    Some(self.entries[pos - 1].1.clone())
                }
            }
        }
    }

    pub fn ease(&self, t: f32, tween: Tween) -> Option<T>
    where
        T: Clone + TryInto<VarValue> + TryFrom<VarValue>,
    {
        if self.entries.is_empty() || t < 0.0 {
            return None;
        }

        // Find the two entries that bracket time t
        match self
            .entries
            .binary_search_by(|(hist_t, _)| hist_t.total_cmp(&t))
        {
            Ok(pos) => {
                // Exact match at time t
                Some(self.entries[pos].1.clone())
            }
            Err(pos) => {
                if pos == 0 {
                    // t is before first entry
                    None
                } else if pos == self.entries.len() {
                    // t is after last entry, return last value
                    Some(self.entries.last().unwrap().1.clone())
                } else {
                    // Interpolate between pos-1 and pos
                    let (t1, v1) = &self.entries[pos - 1];
                    let (t2, v2) = &self.entries[pos];

                    // Try to convert values to VarValue for interpolation
                    let v1_val: Result<VarValue, _> = v1.clone().try_into();
                    let v2_val: Result<VarValue, _> = v2.clone().try_into();

                    if let (Ok(v1_val), Ok(v2_val)) = (v1_val, v2_val) {
                        match tween.f(&v1_val, &v2_val, t - t1, t2 - t1) {
                            Ok(result) => result.try_into().ok(),
                            Err(_) => None,
                        }
                    } else {
                        // If conversion fails, return the earlier value
                        Some(v1.clone())
                    }
                }
            }
        }
    }
}
