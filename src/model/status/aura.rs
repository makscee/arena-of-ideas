use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuraStatus {
    pub distance: Option<Coord>,
    pub clan: Option<Clan>, // TODO: Filter
    pub status: Box<Status>,
}

impl EffectContainer for AuraStatus {
    fn walk_effects_mut(&mut self, f: &mut dyn FnMut(&mut Effect)) {
        self.status.walk_effects_mut(f);
    }
}

impl StatusImpl for AuraStatus {}
