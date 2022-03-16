use super::*;

impl Logic<'_> {
    pub fn process_if_effect(&mut self, QueuedEffect { effect, context }: QueuedEffect<IfEffect>) {
        let condition = match &effect.condition {
            Condition::UnitHasStatus { who, status } => {
                let who = context.get(*who);
                let who = who
                    .and_then(|id| self.model.units.get(&id))
                    .expect("Caster or Target not found");
                who.all_statuses.contains(status)
            }
        };

        let effect = if condition {
            effect.then
        } else {
            effect.r#else
        };
        self.effects.push_back(QueuedEffect { effect, context })
    }
}
