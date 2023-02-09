use super::*;

pub struct NameSystem {}

const EFFECTS_KEY: &str = "names";

impl NameSystem {
    pub fn get_all_shaders(
        world: &legion::World,
        options: &Options,
    ) -> Vec<(legion::Entity, Shader)> {
        <(&EntityComponent, &Name, &Shader)>::query()
            .iter(world)
            .map(|(entity, name, shader)| {
                (
                    entity.entity,
                    options
                        .name
                        .clone()
                        .set_uniform("u_name", ShaderUniform::String(name.0.clone())),
                )
            })
            .collect_vec()
    }

    pub fn fill_node_template(
        world: &legion::World,
        options: &Options,
        node_template: &mut CassetteNode,
    ) {
        node_template.clear_key(EFFECTS_KEY);
        let effects = Self::get_all_shaders(world, options)
            .into_iter()
            .map(|(entity, shader)| {
                VisualEffect::new(
                    0.0,
                    VisualEffectType::EntityExtraShaderConst { entity, shader },
                    0,
                )
            })
            .collect_vec();
        node_template.add_effects_by_key(EFFECTS_KEY, effects);
    }
}
