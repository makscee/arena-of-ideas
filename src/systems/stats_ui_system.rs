use super::*;

pub struct StatsUiSystem {}

impl StatsUiSystem {
    pub fn get_visual_effects(world: &legion::World, resources: &Resources) -> Vec<VisualEffect> {
        <(&HpComponent, &EntityComponent, &Shader)>::query()
            .iter(world)
            .map(|(_, entity, _)| {
                VisualEffect::new(
                    0.0,
                    VisualEffectType::EntityExtraShaderConst {
                        entity: entity.entity,
                        shader: resources
                            .options
                            .stats
                            .clone()
                            .set_uniform("u_offset", ShaderUniform::Float(1.0)),
                    },
                    0,
                )
            })
            .collect_vec()
    }
}
