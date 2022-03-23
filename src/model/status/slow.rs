use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SlowStatus {
    pub percent: f32,
}

impl EffectContainer for SlowStatus {
    fn walk_effects_mut(&mut self, _f: &mut dyn FnMut(&mut Effect)) {}
}

impl StatusImpl for SlowStatus {}
