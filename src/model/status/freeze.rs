use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FreezeStatus {}

impl EffectContainer for FreezeStatus {
    fn walk_effects_mut(&mut self, _f: &mut dyn FnMut(&mut Effect)) {}
}

impl StatusImpl for FreezeStatus {}
