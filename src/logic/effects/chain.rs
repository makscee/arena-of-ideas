pub use super::*;

impl Logic<'_> {
    pub fn process_chain_effect(
        &mut self,
        QueuedEffect { effect, context }: QueuedEffect<ChainEffect>,
    ) {
        let mut touched = HashSet::new();
        let mut touch_effect = effect.effect.clone();
        let mut target = self.model.units.get(&context.target.unwrap()).unwrap();
        let mut effects = Vec::new();
        while touched.len() < effect.targets {
            touched.insert(target.id);
            effects.push(QueuedEffect {
                effect: touch_effect.clone(),
                context: EffectContext {
                    target: Some(target.id),
                    ..context
                },
            });
            touch_effect.apply_modifier(&effect.jump_modifier);
            target = match self
                .model
                .units
                .iter()
                .filter(|unit| unit.faction == target.faction && !touched.contains(&unit.id))
                .filter(|unit| (unit.position - target.position).len() < effect.jump_distance)
                .min_by_key(|unit| (unit.position - target.position).len())
            {
                Some(unit) => unit,
                None => break,
            }
        }
        for effect in effects.into_iter().rev() {
            self.effects.push_front(effect);
        }
    }
}
