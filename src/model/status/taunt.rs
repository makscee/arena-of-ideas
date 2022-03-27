use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TauntStatus {
    pub range: Coord,
}

impl EffectContainer for TauntStatus {
    fn walk_effects_mut(&mut self, _f: &mut dyn FnMut(&mut Effect)) {}
}

impl StatusImpl for TauntStatus {}
