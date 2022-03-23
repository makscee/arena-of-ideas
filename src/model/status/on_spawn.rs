use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OnSpawnStatus {
    pub effect: Effect,
}

impl EffectContainer for OnSpawnStatus {
    fn walk_effects_mut(&mut self, f: &mut dyn FnMut(&mut Effect)) {
        self.effect.walk_mut(f);
    }
}

impl StatusImpl for OnSpawnStatus {}
