use super::*;

pub struct HouseSystem {}

const EFFECTS_KEY_ABILITIES: &str = "ability_descriptions";
impl HouseSystem {
    pub fn get_ability_description_shaders(
        world: &legion::World,
        options: &Options,
        houses: &HashMap<HouseName, House>,
    ) -> Vec<(legion::Entity, Shader)> {
        <(&EntityComponent, &AbilityDescriptionComponent, &Shader)>::query()
            .iter(world)
            .map(|(entity, ability_description, _)| {
                (entity.entity, {
                    let (house_name, ability) = ability_description.0[0].clone();
                    let house = houses.get(&house_name).unwrap();
                    let description = house.abilities.get(&ability).unwrap().description.clone();
                    options
                        .shaders
                        .description_panel
                        .clone()
                        .set_uniform("u_name", ShaderUniform::String((2, ability)))
                        .set_uniform(
                            "u_description",
                            ShaderUniform::String((1, description.clone())),
                        )
                        .set_uniform("u_color", ShaderUniform::Color(house.color))
                        .set_uniform("u_offset", ShaderUniform::Vec2(vec2(1.3, 0.0)))
                })
            })
            .collect_vec()
    }

    pub fn fill_cassette_node_with_descriptions(
        world: &legion::World,
        options: &Options,
        houses: &HashMap<HouseName, House>,
        node: &mut CassetteNode,
    ) {
        node.clear_key(EFFECTS_KEY_ABILITIES);
        let effects = Self::get_ability_description_shaders(world, options, houses)
            .into_iter()
            .map(|(entity, shader)| {
                VisualEffect::new(
                    0.0,
                    VisualEffectType::EntityExtraShaderConst { entity, shader },
                    0,
                )
            })
            .collect_vec();
        node.add_effects_by_key(EFFECTS_KEY_ABILITIES, effects);
    }

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
