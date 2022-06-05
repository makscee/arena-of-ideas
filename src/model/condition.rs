use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum Condition {
    Always,
    Not {
        condition: Box<Condition>,
    },
    UnitHasStatus {
        who: Who,
        #[serde(rename = "status")]
        status_type: StatusName,
    },
    UnitInjured {
        who: Who,
    },
    InRange {
        max_distance: Coord,
    },
    Chance {
        percent: Expr,
    },
}

impl Default for Condition {
    fn default() -> Self {
        Self::Always
    }
}
