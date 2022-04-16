use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OnDealDamageStatus {
    pub damage_type: Option<DamageType>,
    pub effect: Effect,
}

impl EffectContainer for OnDealDamageStatus {
    fn walk_effects_mut(&mut self, f: &mut dyn FnMut(&mut Effect)) {
        self.effect.walk_mut(f);
    }
}

impl StatusImpl for OnDealDamageStatus {}
