use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct FusionData {
    pub units: Vec<String>,
    pub actions: Vec<(UnitTriggerRef, Vec<UnitActionRef>)>,
}

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
