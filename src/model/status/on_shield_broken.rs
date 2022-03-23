use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OnShieldBrokenStatus {
    pub heal: DamageValue,
}

impl EffectContainer for OnShieldBrokenStatus {
    fn walk_effects_mut(&mut self, _f: &mut dyn FnMut(&mut Effect)) {}
}

impl StatusImpl for OnShieldBrokenStatus {}
