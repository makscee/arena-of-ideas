use super::*;

#[derive(Clone, Debug, Deserialize)]
pub struct VisualEffect {
    pub duration: Time,
    #[serde(default)]
    pub delay: Time,
    #[serde(flatten)]
    pub r#type: VisualEffectType,
    #[serde(default)]
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

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum VisualEffectType {
    ShaderAnimation {
        shader: Shader,
        animation: AnimatedShaderUniforms,
    },
    ShaderConst {
        shader: Shader,
    },
    EntityShaderAnimation {
        entity: legion::Entity,
        animation: AnimatedShaderUniforms,
    },
    EntityShaderConst {
        entity: legion::Entity,
        uniforms: ShaderUniforms,
    },
    /// Draw extra shader using and animating uniforms of existing Shader of Entity
    EntityExtraShaderAnimation {
        entity: legion::Entity,
        shader: Shader,
        animation: AnimatedShaderUniforms,
    },
    EntityPairExtraShaderAnimation {
        entity_from: legion::Entity,
        entity_to: legion::Entity,
        shader: Shader,
        animation: AnimatedShaderUniforms,
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
        t: Time,
        entity_shaders: &mut HashMap<legion::Entity, Shader>,
    ) -> Option<Shader> {
        match self {
            VisualEffectType::ShaderAnimation { shader, animation } => {
                Some(shader.clone().merge_uniforms(&animation.get_mixed(t), true))
            }
            VisualEffectType::EntityShaderAnimation { entity, animation } => {
                if let Some(shader) = entity_shaders.get_mut(entity) {
                    shader.merge_uniforms_ref(&animation.get_mixed(t), true);
                }
                None
            }
            VisualEffectType::EntityShaderConst { entity, uniforms } => {
                if let Some(shader) = entity_shaders.get_mut(entity) {
                    shader.merge_uniforms_ref(&uniforms, true);
                }
                None
            }
            VisualEffectType::EntityExtraShaderAnimation {
                entity,
                shader,
                animation,
            } => match entity_shaders.get(entity) {
                Some(entity_shader) => Some(
                    shader
                        .clone()
                        .merge_uniforms(&entity_shader.parameters.uniforms, false)
                        .merge_uniforms(&animation.get_mixed(t), true),
                ),
                _ => None,
            },
            VisualEffectType::EntityPairExtraShaderAnimation {
                entity_from,
                entity_to,
                shader,
                animation,
            } => {
                if let Some(from_shader) = entity_shaders.get(entity_from) {
                    if let Some(to_shader) = entity_shaders.get(entity_to) {
                        let mut shader = shader.clone();
                        let mut uniforms = from_shader
                            .parameters
                            .uniforms
                            .merge(&shader.parameters.uniforms)
                            .merge(&animation.get_mixed(t));
                        uniforms.insert_ref(
                            "u_from",
                            from_shader
                                .parameters
                                .uniforms
                                .get(&VarName::Position.uniform())
                                .cloned()
                                .unwrap(),
                        );
                        uniforms.insert_ref(
                            "u_to",
                            to_shader
                                .parameters
                                .uniforms
                                .get(&VarName::Position.uniform())
                                .cloned()
                                .unwrap(),
                        );
                        shader.parameters.uniforms = uniforms;
                        return Some(shader);
                    }
                }
                None
            }
            VisualEffectType::EntityExtraShaderConst { entity, shader } => {
                match entity_shaders.get(entity) {
                    Some(entity_shader) => Some(
                        shader
                            .clone()
                            .merge_uniforms(&entity_shader.parameters.uniforms, false),
                    ),
                    _ => None,
                }
            }
            VisualEffectType::ShaderConst { shader } => Some(shader.clone()),
        }
    }
}
