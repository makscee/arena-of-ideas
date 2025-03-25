use std::ops::Range;

use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UnitActionRef {
    pub unit: u8,
    pub trigger: u8,
    pub action_range: Range<u8>,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct UnitTriggerRef {
    pub unit: u8,
    pub trigger: u8,
}

impl Default for UnitActionRef {
    fn default() -> Self {
        Self {
            unit: 0,
            trigger: 0,
            action_range: 0..1,
        }
    }
}
