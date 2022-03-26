use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GainedEffectStatus {
    pub effect: Effect,
}

impl EffectContainer for GainedEffectStatus {
    fn walk_effects_mut(&mut self, f: &mut dyn FnMut(&mut Effect)) {
        self.effect.walk_mut(f);
    }
}

impl StatusImpl for GainedEffectStatus {}
