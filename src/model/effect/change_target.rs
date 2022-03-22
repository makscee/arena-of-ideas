use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChangeTargetEffect {
    pub filter: TargetFilter,
    pub condition: Condition,
    pub effect: Effect,
}

impl EffectContainer for ChangeTargetEffect {
    fn walk_effects_mut(&mut self, f: &mut dyn FnMut(&mut Effect)) {
        self.effect.walk_mut(f);
    }
}

impl EffectImpl for ChangeTargetEffect {
    fn process(self: Box<Self>, context: EffectContext, logic: &mut Logic) {
        let effect = *self;
        let caster = context
            .caster
            .and_then(|id| logic.model.units.get(&id))
            .expect("caster not found");
        if let Some(unit) = logic
            .model
            .units
            .iter()
            .filter(|unit| unit.id != caster.id)
            .filter(|unit| effect.filter.matches(unit.faction, caster.faction))
            .filter(|unit| logic.check_condition(&effect.condition, &context))
            .choose(&mut global_rng())
        {
            logic.effects.push_front(QueuedEffect {
                effect: effect.effect.clone(),
                context: EffectContext {
                    target: Some(unit.id),
                    ..context
                },
            });
        }
    }
}
