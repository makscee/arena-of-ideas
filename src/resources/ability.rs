use super::*;

#[derive(Deserialize, Debug, Clone)]
pub struct Ability {
    pub effect: Effect,
    pub description: String,
    #[serde(default)]
    pub vars: Vars,
}
