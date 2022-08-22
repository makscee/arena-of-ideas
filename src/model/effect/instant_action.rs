use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct InstantActionEffect {}

impl EffectContainer for InstantActionEffect {
    fn walk_effects_mut(&mut self, f: &mut dyn FnMut(&mut Effect)) {}
}

impl EffectImpl for InstantActionEffect {
    fn process(self: Box<Self>, context: EffectContext, logic: &mut logic::Logic) {
        let effect = *self;
        let caster = logic.model.units.get(&context.caster.unwrap()).unwrap();
        logic.effects.push_back(QueuedEffect {
            effect: caster.action.effect.clone(),
            context,
        });
    }
}
