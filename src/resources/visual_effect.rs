use super::*;

#[derive(Clone, Debug)]
pub struct VisualEffect {
    pub duration: Time,
    pub r#type: VisualEffectType,
}

#[derive(Clone, Debug)]
pub enum VisualEffectType {
    ShaderAnimation {
        program: PathBuf,
        parameters: ShaderParameters,
        from: ShaderUniforms,
        to: ShaderUniforms,
        easing: EasingType,
    },
    EntityShaderAnimation {
        entity: legion::Entity,
        from: ShaderUniforms,
        to: ShaderUniforms,
        easing: EasingType,
    },
    EntityShaderConst {
        entity: legion::Entity,
        uniforms: ShaderUniforms,
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
                easing,
            } => Some(Shader {
                path: static_path().join(program),
                parameters: ShaderParameters {
                    uniforms: parameters.uniforms.merge(&ShaderUniforms::mix(
                        from,
                        to,
                        easing.f(t),
                    )),
                    ..*parameters
                },
                layer: ShaderLayer::Vfx,
                order: default(),
            }),
            VisualEffectType::EntityShaderAnimation {
                entity,
                from,
                to,
                easing,
            } => {
                if let Some(shader) = entity_shaders.get_mut(entity) {
                    shader.parameters.uniforms = shader
                        .parameters
                        .uniforms
                        .merge(&ShaderUniforms::mix(from, to, easing.f(t)));
                }
                None
            }
            VisualEffectType::EntityShaderConst { entity, uniforms } => {
                if let Some(shader) = entity_shaders.get_mut(entity) {
                    shader.parameters.uniforms = shader.parameters.uniforms.merge(&uniforms);
                }
                None
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum EasingType {
    Linear,
    QuartOut,
    QuartIn,
    QuartInOut,
}

impl EasingType {
    pub fn f(&self, t: f32) -> f32 {
        match self {
            EasingType::Linear => tween::Tweener::linear(0.0, 1.0, 1.0).move_to(t),
            EasingType::QuartOut => tween::Tweener::quart_out(0.0, 1.0, 1.0).move_to(t),
            EasingType::QuartIn => tween::Tweener::quart_in(0.0, 1.0, 1.0).move_to(t),
            EasingType::QuartInOut => tween::Tweener::quart_in_out(0.0, 1.0, 1.0).move_to(t),
        }
    }
}
