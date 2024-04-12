use super::*;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Ability {
    pub name: String,
    pub description: String,
    pub effect: Effect,
}
