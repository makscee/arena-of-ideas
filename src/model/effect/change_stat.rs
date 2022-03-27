use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChangeStatEffect {
    pub stat: UnitStat,
    pub value: Expr,
}

impl EffectContainer for ChangeStatEffect {
    fn walk_effects_mut(&mut self, f: &mut dyn FnMut(&mut Effect)) {}
}

impl EffectImpl for ChangeStatEffect {
    fn process(self: Box<Self>, context: EffectContext, logic: &mut Logic) {
        let effect = *self;
        let value = effect.value.calculate(&context, logic);
        let target = context
            .target
            .and_then(|id| logic.model.units.get_mut(&id))
            .expect("target not found");
        *target.stat_mut(effect.stat) = value;
    }
}
