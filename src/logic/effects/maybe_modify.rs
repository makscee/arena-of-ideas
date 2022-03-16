use super::*;

impl Logic<'_> {
    pub fn process_maybe_modify_effect(
        &mut self,
        QueuedEffect {
            mut effect,
            context,
        }: QueuedEffect<MaybeModifyEffect>,
    ) {
        let condition = match &effect.condition {
            Condition::UnitHasStatus { who, status } => {
                let who = context.get(*who);
                let who = who
                    .and_then(|id| self.model.units.get(&id))
                    .expect("Caster or Target not found");
                who.all_statuses.contains(status)
            }
        };

        if condition {
            effect.base_effect.apply_modifier(&effect.modifier);
        }
        self.effects.push_back(QueuedEffect {
            effect: effect.base_effect,
            context,
        })
    }
}
