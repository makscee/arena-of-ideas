use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SuicideEffect {}

impl SuicideEffect {
    pub fn walk_children_mut(&mut self, _f: &mut impl FnMut(&mut Effect)) {}
}

impl Logic<'_> {
    pub fn process_suicide_effect(
        &mut self,
        QueuedEffect { caster, .. }: QueuedEffect<SuicideEffect>,
    ) {
        if let Some(caster) = caster.and_then(|id| self.model.units.get_mut(&id)) {
            caster.hp = Health::new(0.0);
        }
    }
}
