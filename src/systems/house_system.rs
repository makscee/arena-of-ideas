use super::*;

pub struct HouseSystem {}

impl HouseSystem {
    pub fn get_ability_house(houses: &HashMap<HouseName, House>, ability: &str) -> HouseName {
        houses
            .iter()
            .find_map(
                |(name, house)| match house.abilities.contains_key(ability) {
                    true => Some(*name),
                    false => None,
                },
            )
            .unwrap()
    }
}
