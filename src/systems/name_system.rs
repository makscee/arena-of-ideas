use super::*;

pub struct NameSystem {}

impl NameSystem {
    pub fn get_entity_shader(
        entity: legion::Entity,
        world: &legion::World,
        options: &Options,
    ) -> Shader {
        let name = ContextState::get(entity, world).name.clone();
        options
            .shaders
            .name
            .clone()
            .set_uniform("u_text", ShaderUniform::String((0, name)))
    }
}
