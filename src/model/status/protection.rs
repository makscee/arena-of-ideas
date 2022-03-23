use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProtectionStatus {
    pub percent: f32,
}

impl EffectContainer for ProtectionStatus {
    fn walk_effects_mut(&mut self, _f: &mut dyn FnMut(&mut Effect)) {}
}

impl StatusImpl for ProtectionStatus {}
