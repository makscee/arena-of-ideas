use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModifierStatus {
    pub modifier: Modifier,
}

impl EffectContainer for ModifierStatus {
    fn walk_effects_mut(&mut self, _f: &mut dyn FnMut(&mut Effect)) {}
}

impl StatusImpl for ModifierStatus {}
