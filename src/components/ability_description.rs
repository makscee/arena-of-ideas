use super::*;

#[derive(Deserialize)]
pub struct AbilityDescriptionComponent(pub Vec<(HouseName, String)>);

impl AbilityDescriptionComponent {
    pub fn new(abilities: &Vec<String>, houses: &HashMap<HouseName, House>) -> Self {
        Self(
            abilities
                .iter()
                .map(|ability| {
                    (
                        HouseSystem::get_ability_house(houses, ability),
                        ability.clone(),
                    )
                })
                .collect_vec(),
        )
    }
}
