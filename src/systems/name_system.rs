use super::*;

pub struct NameSystem {}

const EFFECTS_KEY: &str = "names";

impl NameSystem {
    pub fn get_all_shaders(
        world: &legion::World,
        options: &Options,
    ) -> Vec<(legion::Entity, Shader)> {
        <(&EntityComponent, &NameComponent, &Shader)>::query()
            .iter(world)
            .map(|(entity, name, _)| {
                (
                    entity.entity,
                    options
                        .name
                        .clone()
                        .set_uniform("u_name", ShaderUniform::String((1, name.0.clone()))),
                )
            })
            .collect_vec()
    }

    pub fn fill_cassette_node(world: &legion::World, options: &Options, node: &mut CassetteNode) {
        node.clear_key(EFFECTS_KEY);
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
        node.add_effects_by_key(EFFECTS_KEY, effects);
    }
}
