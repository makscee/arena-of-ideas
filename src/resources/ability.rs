use ron::{from_str, to_string};

use super::*;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Ability {
    pub name: String,
    pub description: String,
    pub effect: Effect,
}

impl From<TAbility> for Ability {
    fn from(value: TAbility) -> Self {
        Self {
            name: value.name,
            description: value.description,
            effect: from_str::<Effect>(&value.effect).unwrap(),
        }
    }
}

impl From<Ability> for TAbility {
    fn from(value: Ability) -> Self {
        Self {
            name: value.name,
            description: value.description,
            effect: to_string(&value.effect).unwrap(),
        }
    }
}
