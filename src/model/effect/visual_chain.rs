use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct VisualChainEffect {
    pub effects: Vec<Box<VisualEffect>>,
}

impl EffectContainer for VisualChainEffect {
    fn walk_effects_mut(&mut self, f: &mut dyn FnMut(&mut Effect)) {}
}

impl EffectImpl for VisualChainEffect {
    fn process(self: Box<Self>, context: EffectContext, logic: &mut logic::Logic) {
        let chain_effect = *self;
        let mut delay = Time::new(0.0);
        for mut visual_effect in chain_effect.effects {
            visual_effect.delay += delay;
            delay = visual_effect.delay + visual_effect.duration;
            visual_effect.process(context.clone(), logic);
        }
    }
}
