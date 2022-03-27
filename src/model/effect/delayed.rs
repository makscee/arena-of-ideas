use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct DelayedEffect {
    pub time: Time,
    pub effect: Effect,
}

impl PartialEq for QueuedEffect<DelayedEffect> {
    fn eq(&self, other: &Self) -> bool {
        self.effect.time == other.effect.time
    }
}

impl Eq for QueuedEffect<DelayedEffect> {}

impl PartialOrd for QueuedEffect<DelayedEffect> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.effect.time.cmp(&other.effect.time).reverse())
    }
}

impl Ord for QueuedEffect<DelayedEffect> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.effect.time.cmp(&other.effect.time).reverse()
    }
}

impl EffectContainer for DelayedEffect {
    fn walk_effects_mut(&mut self, f: &mut dyn FnMut(&mut Effect)) {
        self.effect.walk_mut(f);
    }
}

impl EffectImpl for DelayedEffect {
    fn process(self: Box<Self>, context: EffectContext, logic: &mut logic::Logic) {
        let mut effect = *self;
        effect.time += logic.model.time;
        logic
            .model
            .delayed_effects
            .push(QueuedEffect { context, effect });
    }
}
