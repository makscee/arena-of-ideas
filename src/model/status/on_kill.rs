use super::*;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct OnKillStatus {
    pub damage_type: Option<DamageType>,
    pub effect: Effect,
}

impl EffectContainer for OnKillStatus {
    fn walk_effects_mut(&mut self, f: &mut dyn FnMut(&mut Effect)) {
        self.effect.walk_mut(f);
    }
}

impl StatusImpl for OnKillStatus {}
