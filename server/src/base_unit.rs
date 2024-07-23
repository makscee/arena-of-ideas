use rand::seq::IteratorRandom;

use super::*;

#[spacetimedb(table)]
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
    pub fn get_random(houses: &Vec<String>) -> Self {
        Self::iter()
            .filter(|u| u.rarity >= 0 && (houses.is_empty() || houses.contains(&u.house)))
            .choose(&mut thread_rng())
            .unwrap()
    }
}
