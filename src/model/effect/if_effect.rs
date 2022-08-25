use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", deny_unknown_fields, rename = "If")]
pub struct IfEffect {
    pub condition: Condition,
    pub then: Effect,
    pub r#else: Effect,
}

impl EffectContainer for IfEffect {
    fn walk_effects_mut(&mut self, f: &mut dyn FnMut(&mut Effect)) {
        self.then.walk_mut(f);
        self.r#else.walk_mut(f);
    }
}

impl EffectImpl for IfEffect {
    fn process(self: Box<Self>, context: EffectContext, logic: &mut logic::Logic) {
        let effect = *self;
        let effect = if logic.check_condition(&effect.condition, &context) {
            effect.then
        } else {
            effect.r#else
        };
        logic.effects.push_front(QueuedEffect { effect, context });
    }
}
