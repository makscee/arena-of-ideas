use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct RepeatEffect {
    pub times: Expr,
    pub effect: Effect,
}

impl EffectContainer for RepeatEffect {
    fn walk_effects_mut(&mut self, f: &mut dyn FnMut(&mut Effect)) {
        self.effect.walk_mut(f);
    }
}

impl EffectImpl for RepeatEffect {
    fn process(self: Box<Self>, context: EffectContext, logic: &mut logic::Logic) {
        let effect = *self;
        let times = effect.times.calculate(&context, &logic.model);
        for _ in 0..times {
            logic
                .effects
                .push_front(context.clone(), effect.effect.clone())
        }
    }
}
