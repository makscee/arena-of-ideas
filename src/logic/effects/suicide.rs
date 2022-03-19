use super::*;

impl Logic<'_> {
    pub fn process_suicide_effect(
        &mut self,
        QueuedEffect { context, .. }: QueuedEffect<SuicideEffect>,
    ) {
        if let Some(caster) = context.caster.and_then(|id| self.model.units.get_mut(&id)) {
            let caster_id = caster.id;
            self.kill(caster_id);
        }
    }
}
