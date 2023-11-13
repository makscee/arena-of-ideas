use super::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Ability {
    pub name: String,
    pub description: String,
    pub effect: Effect,
}
