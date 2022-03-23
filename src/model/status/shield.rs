use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ShieldStatus {}

impl EffectContainer for ShieldStatus {
    fn walk_effects_mut(&mut self, _f: &mut dyn FnMut(&mut Effect)) {}
}

impl StatusImpl for ShieldStatus {}
