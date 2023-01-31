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
        from: ShaderParameters,
        to: ShaderParameters,
    },
}

impl VisualEffectType {
    pub fn process(&self, t: f32) -> Option<Shader> {
        match self {
            VisualEffectType::ShaderAnimation { program, from, to } => Some(Shader {
                path: static_path().join(program),
                parameters: ShaderParameters::mix(from, to, t),
                layer: ShaderLayer::Vfx,
                order: default(),
            }),
        }
    }
}
