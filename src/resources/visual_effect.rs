use super::*;

#[derive(Clone, Debug)]
pub struct VisualEffect {
    pub duration: Time,
    pub delay: Time,
    pub r#type: VisualEffectType,
    pub order: i32,
}

impl VisualEffect {
    pub fn new(duration: Time, r#type: VisualEffectType, order: i32) -> Self {
        Self::new_delayed(duration, 0.0, r#type, order)
    }

    pub fn new_delayed(duration: Time, delay: Time, r#type: VisualEffectType, order: i32) -> Self {
        Self {
            duration,
            delay,
            r#type,
            order,
        }
    }
}

#[derive(Clone, Debug)]
pub enum VisualEffectType {
    ShaderAnimation {
        shader: Shader,
        from: ShaderUniforms,
        to: ShaderUniforms,
        easing: EasingType,
    },
    ShaderConst {
        shader: Shader,
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
    /// Draw extra shader using and animating uniforms of existing Shader of Entity
    EntityExtraShaderAnimation {
        entity: legion::Entity,
        shader: Shader,
        from: ShaderUniforms,
        to: ShaderUniforms,
        easing: EasingType,
    },
    /// Draw extra shader using uniforms of existing Shader of Entity
    EntityExtraShaderConst {
        entity: legion::Entity,
        shader: Shader,
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
                shader,
                from,
                to,
                easing,
            } => {
                let mut shader = shader.clone();
                shader.parameters = ShaderParameters {
                    uniforms: shader.parameters.uniforms.merge(&ShaderUniforms::mix(
                        from,
                        to,
                        easing.f(t),
                    )),
                    ..shader.parameters
                };
                Some(shader)
            }
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
            VisualEffectType::EntityExtraShaderAnimation {
                entity,
                shader,
                from,
                to,
                easing,
            } => match entity_shaders.get(entity) {
                Some(entity_shader) => {
                    let mut shader = shader.clone();
                    shader.parameters = ShaderParameters {
                        uniforms: entity_shader
                            .parameters
                            .uniforms
                            .merge(&shader.parameters.uniforms)
                            .merge(&ShaderUniforms::mix(from, to, easing.f(t))),
                        ..shader.parameters
                    };
                    Some(shader)
                }
                _ => None,
            },
            VisualEffectType::EntityExtraShaderConst { entity, shader } => {
                match entity_shaders.get(entity) {
                    Some(entity_shader) => {
                        let mut shader = shader.clone();
                        shader.parameters = ShaderParameters {
                            uniforms: entity_shader
                                .parameters
                                .uniforms
                                .merge(&shader.parameters.uniforms),
                            ..shader.parameters
                        };
                        Some(shader)
                    }
                    _ => None,
                }
            }
            VisualEffectType::ShaderConst { shader } => Some(shader.clone()),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum EasingType {
    Linear,
    QuartOut,
    QuartIn,
    QuartInOut,
    CubicIn,
    BackIn,
}

impl EasingType {
    pub fn f(&self, t: f32) -> f32 {
        match self {
            EasingType::Linear => tween::Tweener::linear(0.0, 1.0, 1.0).move_to(t),
            EasingType::QuartOut => tween::Tweener::quart_out(0.0, 1.0, 1.0).move_to(t),
            EasingType::QuartIn => tween::Tweener::quart_in(0.0, 1.0, 1.0).move_to(t),
            EasingType::QuartInOut => tween::Tweener::quart_in_out(0.0, 1.0, 1.0).move_to(t),
            EasingType::CubicIn => tween::Tweener::cubic_in(0.0, 1.0, 1.0).move_to(t),
            EasingType::BackIn => tween::Tweener::back_in(0.0, 1.0, 1.0).move_to(t),
        }
    }
}
