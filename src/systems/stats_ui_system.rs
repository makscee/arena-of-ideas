use super::*;

pub struct StatsUiSystem {}

const STATS_EFFECTS_KEY: &str = "stats";

#[derive(Clone, Deserialize, Eq, PartialEq, Debug)]
pub enum StatType {
    Hp,
    Attack,
}

impl StatsUiSystem {
    pub fn get_all_shaders(
        world: &legion::World,
        options: &Options,
        stat: StatType,
    ) -> Vec<(legion::Entity, Shader)> {
        match stat {
            StatType::Hp => <(&HpComponent, &EntityComponent, &Shader)>::query()
                .iter(world)
                .map(|(hp, entity, unit_shader)| {
                    (entity.entity, {
                        let mut shader = options
                            .shaders
                            .stats
                            .clone()
                            .set_uniform("u_offset", ShaderUniform::Float(1.0))
                            .set_uniform(
                                "u_text",
                                ShaderUniform::String((0, hp.current.to_string())),
                            )
                            .set_uniform(
                                "u_value_modified",
                                ShaderUniform::Int(hp.current - hp.max),
                            )
                            .set_uniform(
                                "u_circle_color",
                                ShaderUniform::Color(options.colors.stats_hp_color),
                            );
                        shader.parameters.uniforms = unit_shader
                            .parameters
                            .uniforms
                            .merge(&shader.parameters.uniforms);
                        shader
                    })
                })
                .collect_vec(),
            StatType::Attack => <(&AttackComponent, &EntityComponent, &Shader)>::query()
                .iter(world)
                .map(|(attack, entity, unit_shader)| {
                    (entity.entity, {
                        let mut shader = options
                            .shaders
                            .stats
                            .clone()
                            .set_uniform("u_offset", ShaderUniform::Float(-1.0))
                            .set_uniform(
                                "u_text",
                                ShaderUniform::String((0, attack.value.to_string())),
                            )
                            .set_uniform(
                                "u_value_modified",
                                ShaderUniform::Int(attack.value - attack.initial_value),
                            )
                            .set_uniform("u_damage_taken", ShaderUniform::Float(0.0))
                            .set_uniform(
                                "u_circle_color",
                                ShaderUniform::Color(options.colors.stats_attack_color),
                            );
                        shader.parameters.uniforms = unit_shader
                            .parameters
                            .uniforms
                            .merge(&shader.parameters.uniforms);
                        shader
                    })
                })
                .collect_vec(),
        }
    }

    pub fn fill_cassette_node(world: &legion::World, options: &Options, node: &mut CassetteNode) {
        node.clear_key(STATS_EFFECTS_KEY);
        let effects = Self::get_all_shaders(world, options, StatType::Hp)
            .into_iter()
            .chain(Self::get_all_shaders(world, options, StatType::Attack))
            .map(|(entity, shader)| {
                VisualEffect::new(
                    0.0,
                    VisualEffectType::EntityExtraShaderConst { entity, shader },
                    0,
                )
            })
            .collect_vec();
        node.add_effects_by_key(STATS_EFFECTS_KEY, effects);
    }
}
