use super::*;

pub struct NameSystem {}

const EFFECTS_KEY: &str = "names";

impl NameSystem {
    pub fn get_all_shaders(
        world: &legion::World,
        resources: &Resources,
    ) -> Vec<(legion::Entity, Shader)> {
        <(&EntityComponent, &Name, &Shader)>::query()
            .iter(world)
            .map(|(entity, name, shader)| {
                (
                    entity.entity,
                    resources
                        .options
                        .name
                        .clone()
                        .set_uniform("u_name", ShaderUniform::String(name.0.clone())),
                )
            })
            .collect_vec()
    }

    pub fn fill_node_template(world: &legion::World, resources: &mut Resources) {
        resources.cassette.node_template.clear_key(EFFECTS_KEY);
        let effects = Self::get_all_shaders(world, resources)
            .into_iter()
            .map(|(entity, shader)| {
                VisualEffect::new(
                    0.0,
                    VisualEffectType::EntityExtraShaderConst { entity, shader },
                    0,
                )
            })
            .collect_vec();
        resources
            .cassette
            .node_template
            .add_effects_by_key(EFFECTS_KEY, effects);
    }
}
