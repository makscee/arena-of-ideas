use super::*;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct OnHealStatus {
    #[serde(flatten)]
    pub effect: Effect,
}

impl EffectContainer for OnHealStatus {
    fn walk_effects_mut(&mut self, f: &mut dyn FnMut(&mut Effect)) {
        self.effect.walk_mut(f);
    }
}

impl StatusImpl for OnHealStatus {}
