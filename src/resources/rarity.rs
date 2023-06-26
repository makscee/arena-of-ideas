use strum_macros::Display;

use super::*;

#[derive(
    Clone,
    Copy,
    Deserialize,
    Serialize,
    Debug,
    Eq,
    PartialEq,
    Hash,
    enum_iterator::Sequence,
    Display,
)]
pub enum Rarity {
    Common,
    Rare,
    Epic,
    Legendary,
}

impl Rarity {
    pub fn color(&self, resources: &Resources) -> Rgba<f32> {
        match self {
            Rarity::Common => resources.options.colors.common,
            Rarity::Rare => resources.options.colors.rare,
            Rarity::Epic => resources.options.colors.epic,
            Rarity::Legendary => resources.options.colors.legendary,
        }
    }

    pub fn weight(&self) -> i32 {
        match self {
            Rarity::Common => 100,
            Rarity::Rare => 15,
            Rarity::Epic => 7,
            Rarity::Legendary => 3,
        }
    }
}
