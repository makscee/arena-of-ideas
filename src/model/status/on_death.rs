use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OnDeathStatus {
    pub effect: Effect,
}

impl EffectContainer for OnDeathStatus {
    fn walk_effects_mut(&mut self, f: &mut dyn FnMut(&mut Effect)) {
        self.effect.walk_mut(f);
    }
}

impl StatusImpl for OnDeathStatus {}
