use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Hash)]
#[serde(deny_unknown_fields)]
pub struct Reaction {
    pub trigger: Trigger,
    pub actions: Actions,
}
