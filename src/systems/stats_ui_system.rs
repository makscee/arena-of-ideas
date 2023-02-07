use super::*;

pub struct StatsUiSystem {}

const STATS_EFFECTS_KEY: &str = "stats";

impl StatsUiSystem {
    pub fn add_all_units_stats_to_node_template(world: &legion::World, resources: &mut Resources) {
        resources
            .cassette
            .node_template
            .clear_key(STATS_EFFECTS_KEY);
        resources.cassette.node_template.add_effects_by_key(
            STATS_EFFECTS_KEY,
            <(&HpComponent, &EntityComponent, &Shader)>::query()
                .iter(world)
                .map(|(hp, entity, _)| {
                    VisualEffect::new(
                        0.0,
                        VisualEffectType::EntityExtraShaderConst {
                            entity: entity.entity,
                            shader: resources
                                .options
                                .stats
                                .clone()
                                .set_uniform("u_offset", ShaderUniform::Float(1.0))
                                .set_uniform(
                                    "u_hp",
                                    ShaderUniform::String(hp.current().to_string()),
                                )
                                .set_uniform("u_last_change", ShaderUniform::Float(hp.last_change))
                                .set_uniform(
                                    "u_value_modified",
                                    ShaderUniform::Int(hp.current() - hp.max),
                                )
                                .set_uniform(
                                    "u_circle_color",
                                    ShaderUniform::Color(resources.options.stats_hp_color),
                                ),
                        },
                        0,
                    )
                })
                .collect_vec(),
        );
        resources.cassette.node_template.add_effects_by_key(
            STATS_EFFECTS_KEY,
            <(&AttackComponent, &EntityComponent, &Shader)>::query()
                .iter(world)
                .map(|(attack, entity, _)| {
                    VisualEffect::new(
                        0.0,
                        VisualEffectType::EntityExtraShaderConst {
                            entity: entity.entity,
                            shader: resources
                                .options
                                .stats
                                .clone()
                                .set_uniform("u_offset", ShaderUniform::Float(-1.0))
                                .set_uniform(
                                    "u_attack",
                                    ShaderUniform::String(attack.value().to_string()),
                                )
                                .set_uniform(
                                    "u_last_change",
                                    ShaderUniform::Float(attack.last_change()),
                                )
                                .set_uniform(
                                    "u_value_modified",
                                    ShaderUniform::Int(attack.value() - attack.initial_value()),
                                )
                                .set_uniform(
                                    "u_circle_color",
                                    ShaderUniform::Color(resources.options.stats_attack_color),
                                ),
                        },
                        0,
                    )
                })
                .collect_vec(),
        );
    }
}
