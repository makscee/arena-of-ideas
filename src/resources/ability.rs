use super::*;

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Ability {
    pub effect: EffectWrapped,
    pub description: String,
    #[serde(default)]
    pub default_vars: Vars,
}
