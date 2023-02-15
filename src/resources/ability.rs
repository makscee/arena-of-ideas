use super::*;

#[derive(Deserialize, Debug, Clone)]
pub struct Ability {
    pub effect: Effect,
    #[serde(default)]
    pub vars: Vars,
}
