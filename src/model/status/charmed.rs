use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CharmedStatus {
    pub master_faction: Faction,
}

impl EffectContainer for CharmedStatus {
    fn walk_effects_mut(&mut self, _f: &mut dyn FnMut(&mut Effect)) {}
}

impl StatusImpl for CharmedStatus {}
