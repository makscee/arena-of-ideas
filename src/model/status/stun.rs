use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StunStatus {}

impl EffectContainer for StunStatus {
    fn walk_effects_mut(&mut self, _f: &mut dyn FnMut(&mut Effect)) {}
}

impl StatusImpl for StunStatus {}
