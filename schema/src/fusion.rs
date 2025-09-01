use super::*;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default, Hash)]
pub struct UnitActionRange {
    pub trigger: u8,
    pub start: u8,
    pub length: u8,
}
