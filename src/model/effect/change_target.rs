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

        // This solution seems error-prone in case we forget to consider `Charmed` status at any point
        // or use `unit.faction` instead of `unit_faction`
        // The same code is used in the initial targetting
        let caster_faction = caster
            .attached_statuses
            .iter()
            .find_map(|status| match &status.status {
                Status::Charmed(charm) => status
                    .caster
                    .and_then(|id| logic.model.units.get(&id).map(|unit| unit.faction)),
                _ => None,
            })
            .unwrap_or(caster.faction);

        if let Some(unit) = logic
            .model
            .units
            .iter()
            .filter(|unit| unit.id != caster.id && Some(unit.id) != context.target)
            .filter(|unit| effect.filter.matches(unit.faction, caster_faction))
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
