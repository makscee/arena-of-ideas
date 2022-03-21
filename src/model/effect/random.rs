use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WeightedEffect {
    pub weight: f32,
    #[serde(flatten)]
    pub effect: Effect,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct RandomEffect {
    pub choices: Vec<WeightedEffect>,
}

impl EffectContainer for RandomEffect {
    fn walk_effects_mut(&mut self, f: &mut dyn FnMut(&mut Effect)) {
        for choice in &mut self.choices {
            choice.effect.walk_mut(f);
        }
    }
}

impl EffectImpl for RandomEffect {
    fn process(self: Box<Self>, context: EffectContext, logic: &mut logic::Logic) {
        let effect = *self;
        let effect = effect
            .choices
            .choose_weighted(&mut global_rng(), |choice| choice.weight)
            .unwrap()
            .effect
            .clone();
        logic.effects.push_front(QueuedEffect { effect, context });
    }
}
