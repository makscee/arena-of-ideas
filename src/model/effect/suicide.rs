use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SuicideEffect {}

impl EffectContainer for SuicideEffect {
    fn walk_effects_mut(&mut self, _f: &mut dyn FnMut(&mut Effect)) {}
}

impl EffectImpl for SuicideEffect {
    fn process(self: Box<Self>, context: EffectContext, logic: &mut logic::Logic) {
        let effect = *self;
        if let Some(caster) = context.caster.and_then(|id| logic.model.units.get_mut(&id)) {
            let caster_id = caster.id;
            logic.kill(caster_id);
        }
    }
}
