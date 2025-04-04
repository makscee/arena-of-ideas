use super::*;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct UnitActionRef {
    pub unit: u8,
    pub trigger: u8,
    pub action: u8,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct UnitTriggerRef {
    pub unit: u8,
    pub trigger: u8,
}
