use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum Condition {
    Always,
    UnitHasStatus {
        who: Who,
        #[serde(rename = "status")]
        status_type: StatusType,
    },
    UnitInjured {
        who: Who,
    },
    InRange {
        max_distance: Coord,
    },
}

impl Default for Condition {
    fn default() -> Self {
        Self::Always
    }
}
