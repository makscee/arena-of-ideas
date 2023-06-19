use super::*;

#[derive(Clone, Debug, Deserialize)]
pub struct TimedEffect {
    pub duration: Option<Time>,
    #[serde(default)]
    pub delay: Time,
    #[serde(flatten)]
    pub animation: Animation,
    #[serde(default)]
    pub order: i32,
}

impl TimedEffect {
    pub fn new(duration: Option<Time>, animation: Animation, order: i32) -> Self {
        Self::new_delayed(duration, 0.0, animation, order)
    }

    pub fn new_delayed(
        duration: Option<Time>,
        delay: Time,
        animation: Animation,
        order: i32,
    ) -> Self {
        Self {
            duration,
            delay,
            animation,
            order,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Animation {
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

impl Animation {
    pub fn update_entities(&self, t: Time, entity_shaders: &mut HashMap<legion::Entity, Shader>) {
        match self {
            Animation::EntityShaderAnimation { entity, animation } => {
                if let Some(shader) = entity_shaders.get_mut(entity) {
                    shader.merge_uniforms_ref(&animation.get_mixed(t), true);
                }
            }
            Animation::EntityShaderConst { entity, uniforms } => {
                if let Some(shader) = entity_shaders.get_mut(entity) {
                    shader.merge_uniforms_ref(&uniforms, true);
                }
            }
            _ => {}
        }
    }

    pub fn generate_shaders(
        &self,
        t: Time,
        entity_shaders: &HashMap<legion::Entity, Shader>,
    ) -> Option<Shader> {
        match self {
            Animation::ShaderAnimation { shader, animation } => {
                Some(shader.clone().merge_uniforms(&animation.get_mixed(t), true))
            }
            Animation::EntityExtraShaderAnimation {
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
            Animation::EntityPairExtraShaderAnimation {
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
                            "u_from".to_owned(),
                            from_shader.parameters.uniforms.get("u_position").unwrap(),
                        );
                        uniforms.insert_ref(
                            "u_to".to_owned(),
                            to_shader.parameters.uniforms.get("u_position").unwrap(),
                        );
                        shader.parameters.uniforms = uniforms;
                        return Some(shader);
                    }
                }
                None
            }
            Animation::EntityExtraShaderConst { entity, shader } => {
                match entity_shaders.get(entity) {
                    Some(entity_shader) => Some(
                        shader
                            .clone()
                            .merge_uniforms(&entity_shader.parameters.uniforms, false),
                    ),
                    _ => None,
                }
            }
            Animation::ShaderConst { shader } => Some(shader.clone()),
            _ => None,
        }
    }
}
