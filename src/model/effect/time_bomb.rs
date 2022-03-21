use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TimeBombEffect {
    pub time: Time,
    pub effect: Effect,
}

impl EffectContainer for TimeBombEffect {
    fn walk_effects_mut(&mut self, f: &mut dyn FnMut(&mut Effect)) {
        self.effect.walk_mut(f);
    }
}

impl EffectImpl for TimeBombEffect {
    fn process(self: Box<Self>, context: EffectContext, logic: &mut logic::Logic) {
        let effect = *self;
        let target = context
            .target
            .and_then(|id| {
                logic
                    .model
                    .units
                    .get(&id)
                    .or(logic.model.dead_units.get(&id))
            })
            .expect("Target not found");
        logic.model.time_bombs.insert(TimeBomb {
            id: logic.model.next_id,
            position: target.position,
            caster: context.caster,
            time: effect.time,
            effect: effect.effect,
        });
        logic.model.next_id += 1;
    }
}
