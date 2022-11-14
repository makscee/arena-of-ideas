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
    fn process(self: Box<Self>, mut context: EffectContext, logic: &mut Logic) {
        let effect = *self;
        let owner = logic.model.get(Who::Owner, &context);

        let owner_faction = owner.faction;
        let target = context.target;
        if let Some(unit) = logic
            .model
            .units
            .iter()
            .filter(|unit| unit.id != owner.id && unit.id != target)
            .filter(|unit| effect.filter.matches(unit.faction, owner_faction))
            .filter(|unit| {
                context.target = unit.id;
                Logic::check_condition(&logic.model, &effect.condition, &context)
            })
            .choose(&mut global_rng())
        {
            context.target = unit.id;
            logic.effects.push_front(context, effect.effect);
        }
    }
}
