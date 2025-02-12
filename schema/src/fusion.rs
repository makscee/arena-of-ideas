use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UnitActionRef {
    pub unit: u8,
    pub trigger: u8,
    pub action: u8,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnitTriggerRef {
    pub unit: u8,
    pub trigger: u8,
}
