use itertools::Itertools;
use rand::{
    distributions::{Distribution, WeightedIndex},
    Rng,
};

use super::*;

#[spacetimedb(table(public))]
pub struct TBaseUnit {
    #[primarykey]
    pub name: String,
    pub pwr: i32,
    pub hp: i32,
    pub rarity: i8,
    pub house: String,
    pub triggers: Vec<String>,
    pub targets: Vec<String>,
    pub effects: Vec<String>,
}

impl TBaseUnit {
    pub fn get_random(houses: &Vec<String>, weights: &Vec<i32>, rng: &mut impl Rng) -> Self {
        let mut units = Self::iter()
            .filter(|u| u.rarity >= 0 && (houses.is_empty() || houses.contains(&u.house)))
            .collect_vec();
        let dist = WeightedIndex::new(units.iter().map(|u| weights[u.rarity as usize])).unwrap();
        units.remove(dist.sample(rng))
    }
    pub fn get_random_for_lootbox() -> Self {
        Self::get_random(
            &[].into(),
            &GlobalSettings::get().rarities.lootbox_weights,
            &mut rng(),
        )
    }
}
