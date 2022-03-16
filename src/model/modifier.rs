use super::*;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct StrengthModifier {
    pub multiplier: R32,
    pub add: R32,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum Modifier {
    Strength(StrengthModifier),
}
