use geng::prelude::itertools::Itertools;

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

impl BonusEffect {
    pub fn new_buff_effect(rarity: Rarity, resources: &Resources) -> Self {
        let mut single_target = false;
        let rng = &mut thread_rng();
        let (effect, description) = if rarity == Rarity::Legendary && rng.gen_bool(0.5) {
            let buff = BuffPool::random_team_buff(resources);
            Self::get_team_status(buff)
        } else {
            let (name, mut charges) = BuffPool::random_unit_buff(resources);
            let name = name.to_owned();
            match rarity {
                Rarity::Common => {
                    single_target = true;
                    Self::get_buff_single(charges, name)
                }
                Rarity::Rare => {
                    single_target = true;
                    charges *= 3;
                    Self::get_buff_single(charges, name)
                }
                Rarity::Epic => {
                    if rng.gen_bool(0.5) {
                        single_target = true;
                        charges *= 5;
                        Self::get_buff_single(charges, name)
                    } else {
                        Self::get_buff_team(charges, name)
                    }
                }
                Rarity::Legendary => {
                    if rng.gen_bool(0.5) {
                        single_target = true;
                        charges *= 10;
                        Self::get_buff_single(charges, name)
                    } else {
                        charges *= 3;
                        Self::get_buff_team(charges, name)
                    }
                }
            }
        };
        Self {
            effect,
            rarity,
            description,
            single_target,
            target: None,
        }
    }

    fn get_buff_single(charges: i32, name: String) -> (EffectWrapped, String) {
        (
            Effect::ChangeStatus {
                name: name.to_owned(),
                charges: ExpressionInt::Const { value: charges },
            }
            .wrap(),
            format!("Add {} ({})", name, charges),
        )
    }

    fn get_buff_team(charges: i32, name: String) -> (EffectWrapped, String) {
        let description = format!("Add {} ({}) to everyone", name, charges);
        let effect = Box::new(
            Effect::ChangeStatus {
                name,
                charges: ExpressionInt::Const { value: charges },
            }
            .wrap(),
        );
        let effect = Effect::Aoe {
            factions: vec![ExpressionFaction::Team],
            effect,
            exclude_self: false,
        }
        .wrap();
        (effect, description)
    }

    fn get_team_status(buff: &TeamBuff) -> (EffectWrapped, String) {
        let mut effects = Vec::default();
        for (name, charges) in buff.statuses.iter() {
            for _ in 0..*charges {
                effects.push(
                    Effect::AddTeamStatus {
                        name: name.to_owned(),
                    }
                    .wrap(),
                )
            }
        }
        let effect = Effect::List {
            items: effects.into_iter().map(|x| Box::new(x)).collect_vec(),
        }
        .wrap();
        (effect, format!("Add Team status {}", buff.prefix))
    }

    pub fn new_slot_effect(rarity: Rarity) -> Self {
        let value: i32 = match rarity {
            Rarity::Common | Rarity::Rare | Rarity::Epic => 1,
            Rarity::Legendary => 2,
        };
        let effect = Effect::ChangeTeamVarInt {
            var: VarName::Slots,
            delta: ExpressionInt::Const { value },
            faction: Some(ExpressionFaction::Team),
        }
        .wrap();
        let description = format!("+{value} slots");
        Self {
            effect,
            rarity,
            description,
            single_target: default(),
            target: default(),
        }
    }
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

impl Rarity {
    pub fn color(&self, resources: &Resources) -> Rgba<f32> {
        *resources.options.colors.rarities.get(self).unwrap()
    }

    pub fn weight(&self) -> i32 {
        match self {
            Rarity::Common => 100,
            Rarity::Rare => 15,
            Rarity::Epic => 7,
            Rarity::Legendary => 3,
        }
    }

    pub fn generate(&self, buff: bool, resources: &Resources) -> BonusEffect {
        match buff {
            true => BonusEffect::new_buff_effect(*self, resources),
            false => BonusEffect::new_slot_effect(*self),
        }
    }
}
