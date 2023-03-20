use super::*;

#[derive(Deserialize, Debug, Clone)]
pub struct Ability {
    pub effect: EffectWrapped,
    pub description: String,
    #[serde(default)]
    pub vars: Vars,
}
