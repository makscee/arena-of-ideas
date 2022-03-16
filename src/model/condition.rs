use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum Condition {
    Always,
    UnitHasStatus { who: Who, status: Status },
}

impl Default for Condition {
    fn default() -> Self {
        Self::Always
    }
}
