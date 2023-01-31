use super::*;

#[derive(Clone)]
pub struct VisualEffect {
    pub duration: Time,
    pub r#type: VisualEffectType,
}

#[derive(Clone)]
pub enum VisualEffectType {
    ShaderAnimation {
        program: PathBuf,
        parameters: ShaderParameters,
        from: ShaderUniforms,
        to: ShaderUniforms,
    },
    EntityShaderAnimation {
        entity: legion::Entity,
        from: ShaderUniforms,
        to: ShaderUniforms,
    },
}

impl VisualEffectType {
    pub fn process(
        &self,
        t: f32,
        entity_shaders: &mut HashMap<legion::Entity, Shader>,
    ) -> Option<Shader> {
        match self {
            VisualEffectType::ShaderAnimation {
                program,
                parameters,
                from,
                to,
            } => Some(Shader {
                path: static_path().join(program),
                parameters: ShaderParameters {
                    uniforms: parameters.uniforms.merge(&ShaderUniforms::mix(from, to, t)),
                    ..*parameters
                },
                layer: ShaderLayer::Vfx,
                order: default(),
            }),
            VisualEffectType::EntityShaderAnimation { entity, from, to } => {
                if let Some(shader) = entity_shaders.get_mut(entity) {
                    shader.parameters.uniforms = shader
                        .parameters
                        .uniforms
                        .merge(&ShaderUniforms::mix(from, to, t));
                }
                None
            }
        }
    }
}
