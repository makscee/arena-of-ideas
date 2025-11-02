use super::*;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct UnitActionRange {
    pub start: u8,
    pub length: u8,
}

impl Default for UnitActionRange {
    fn default() -> Self {
        Self {
            start: 0,
            length: u8::MAX,
        }
    }
}
