use super::*;

pub struct StatsUiSystem {}

#[derive(Clone, Deserialize, Eq, PartialEq, Debug)]
pub enum StatType {
    Hp,
    Attack,
}

impl StatsUiSystem {
    pub fn get_entity_shaders(
        entity: legion::Entity,
        world: &legion::World,
        options: &Options,
    ) -> Vec<Shader> {
        world
            .entry_ref(entity)
            .unwrap()
            .get_component::<HpComponent>()
            .map(|hp| {
                options
                    .shaders
                    .stats
                    .clone()
                    .set_uniform("u_angle_offset", ShaderUniform::Float(1.0))
                    .set_uniform("u_text", ShaderUniform::String((0, hp.current.to_string())))
                    .set_uniform("u_value_modified", ShaderUniform::Int(hp.current - hp.max))
                    .set_uniform(
                        "u_circle_color",
                        ShaderUniform::Color(options.colors.stats_hp_color),
                    )
            })
            .into_iter()
            .chain(
                world
                    .entry_ref(entity)
                    .unwrap()
                    .get_component::<AttackComponent>()
                    .map(|attack| {
                        options
                            .shaders
                            .stats
                            .clone()
                            .set_uniform("u_angle_offset", ShaderUniform::Float(-1.0))
                            .set_uniform(
                                "u_text",
                                ShaderUniform::String((0, attack.value.to_string())),
                            )
                            .set_uniform(
                                "u_value_modified",
                                ShaderUniform::Int(attack.value - attack.initial_value),
                            )
                            .set_uniform(
                                "u_circle_color",
                                ShaderUniform::Color(options.colors.stats_attack_color),
                            )
                    })
                    .into_iter(),
            )
            .collect_vec()
    }
}
