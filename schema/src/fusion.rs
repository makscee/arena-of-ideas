use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct FusedUnit {
    pub units: Vec<String>,
    pub triggers: Vec<u8>,
    pub actions: Vec<(u8, u8)>, // unit index, action index
}
