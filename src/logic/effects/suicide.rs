use super::*;

impl Game {
    pub fn process_suicide_effect(
        &mut self,
        QueuedEffect { caster, .. }: QueuedEffect<SuicideEffect>,
    ) {
        if let Some(caster) = caster.and_then(|id| self.units.get_mut(&id)) {
            caster.hp = Health::new(0.0);
        }
    }
}
