use super::*;

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum Trigger {
    AfterTakeDamage { effect: Effect },
}
