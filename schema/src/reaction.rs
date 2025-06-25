use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Hash)]
#[serde(deny_unknown_fields)]
pub struct Reaction {
    pub trigger: Trigger,
    pub actions: Vec<Action>,
}

pub trait ReactionTier {
    fn tier(&self) -> u8;
}

impl ReactionTier for Vec<Reaction> {
    fn tier(&self) -> u8 {
        let action_tiers = self
            .iter()
            .map(|r| r.actions.iter().map(|a| a.tier()).sum::<u8>())
            .sum::<u8>();
        (action_tiers + 1) / 2
    }
}
