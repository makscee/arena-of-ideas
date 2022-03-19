use super::*;

impl Logic<'_> {
    pub fn process_instant_action(&mut self, context: EffectContext) {
        let caster = self.model.units.get(&context.caster.unwrap()).unwrap();
        self.effects.push_back(QueuedEffect {
            effect: caster.action.effect.clone(),
            context,
        });
    }
}
