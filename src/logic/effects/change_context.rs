use super::*;

impl Logic<'_> {
    pub fn process_change_context_effect(
        &mut self,
        QueuedEffect { effect, context }: QueuedEffect<ChangeContextEffect>,
    ) {
        self.effects.push_front(QueuedEffect {
            effect: effect.effect,
            context: EffectContext {
                caster: match effect.caster {
                    Some(who) => context.get(who),
                    None => context.caster,
                },
                from: match effect.from {
                    Some(who) => context.get(who),
                    None => context.from,
                },
                target: match effect.target {
                    Some(who) => context.get(who),
                    None => context.target,
                },
            },
        });
    }
}
