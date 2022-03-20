use super::*;

impl Logic<'_> {
    pub fn process_if_effect(&mut self, QueuedEffect { effect, context }: QueuedEffect<IfEffect>) {
        let effect = if self.check_condition(&effect.condition, &context) {
            effect.then
        } else {
            effect.r#else
        };
        self.effects.push_front(QueuedEffect { effect, context })
    }
}
