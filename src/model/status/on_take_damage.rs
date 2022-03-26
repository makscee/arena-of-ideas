use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OnTakeDamageStatus {
    pub damage_type: Option<DamageType>,
    pub effect: Effect,
}

impl EffectContainer for OnTakeDamageStatus {
    fn walk_effects_mut(&mut self, f: &mut dyn FnMut(&mut Effect)) {
        self.effect.walk_mut(f);
    }
}

impl StatusImpl for OnTakeDamageStatus {}
