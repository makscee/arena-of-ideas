use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AttackSpeedStatus {
    pub percent: f32,
}

impl EffectContainer for AttackSpeedStatus {
    fn walk_effects_mut(&mut self, _f: &mut dyn FnMut(&mut Effect)) {}
}

impl StatusImpl for AttackSpeedStatus {}
