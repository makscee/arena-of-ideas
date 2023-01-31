use super::*;

pub struct VisualEffect {
    pub duration: Time,
    pub r#type: VisualEffectType,
}

pub enum VisualEffectType {
    ShaderAnimation {
        program: PathBuf,
        from: ShaderParameters,
        to: ShaderParameters,
    },
}

impl VisualEffectType {
    pub fn process(&self, t: f32, resources: &Resources) -> Option<Shader> {
        match self {
            VisualEffectType::ShaderAnimation { program, from, to } => Some(Shader {
                path: program.clone(),
                parameters: ShaderParameters::mix(from, to, t),
                layer: ShaderLayer::Vfx,
                order: default(),
            }),
        }
    }
}
