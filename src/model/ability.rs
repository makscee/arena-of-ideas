use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Ability {
    pub effect: Effect,
    pub cooldown: Time,
}
