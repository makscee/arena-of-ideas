use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", deny_unknown_fields)]
pub struct MaybeModifyEffect {
    pub base_effect: Effect,
    pub condition: Condition,
    pub modifier: Modifier,
}

impl EffectContainer for MaybeModifyEffect {
    fn walk_effects_mut(&mut self, f: &mut dyn FnMut(&mut Effect)) {
        self.base_effect.walk_mut(f);
    }
}

impl EffectImpl for MaybeModifyEffect {
    fn process(self: Box<Self>, context: EffectContext, logic: &mut logic::Logic) {
        let mut effect = *self;
        if logic.check_condition(&effect.condition, &context) {
            effect.base_effect.apply_modifier(&effect.modifier);
        }
        logic.effects.push_front(QueuedEffect {
            effect: effect.base_effect,
            context,
        });
    }
}
