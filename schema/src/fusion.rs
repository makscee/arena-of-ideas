use super::*;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct UnitActionRange {
    pub trigger: u8,
    pub start: u8,
    pub length: u8,
}

impl Default for UnitActionRange {
    fn default() -> Self {
        Self {
            trigger: 0,
            start: 0,
            length: u8::MAX,
        }
    }
}
