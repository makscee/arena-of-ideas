use super::*;

impl Logic<'_> {
    pub fn process_maybe_modify_effect(
        &mut self,
        QueuedEffect {
            mut effect,
            context,
        }: QueuedEffect<MaybeModifyEffect>,
    ) {
        if self.check_condition(&effect.condition, &context) {
            effect.base_effect.apply_modifier(&effect.modifier);
        }
        self.effects.push_back(QueuedEffect {
            effect: effect.base_effect,
            context,
        })
    }
}
