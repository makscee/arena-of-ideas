use super::*;

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct BonusEffect {
    pub effect: EffectWrapped,
    pub rarity: Rarity,
    pub description: String,
    #[serde(default)]
    pub single_target: bool,
    #[serde(skip)]
    pub target: Option<(legion::Entity, String)>,
}

#[derive(
    Clone, Copy, Deserialize, Serialize, Debug, Eq, PartialEq, Hash, enum_iterator::Sequence,
)]
pub enum Rarity {
    Common,
    Rare,
    Epic,
    Legendary,
}
