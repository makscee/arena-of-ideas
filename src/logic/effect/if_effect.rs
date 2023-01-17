use super::*;

use crate::model::Condition;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", deny_unknown_fields, rename = "If")]
pub struct IfEffect {
    pub condition: Condition,
    pub then: LogicEffect,
    pub r#else: LogicEffect,
}

impl EffectContainer for IfEffect {
    fn walk_effects_mut(&mut self, f: &mut dyn FnMut(&mut LogicEffect)) {
        self.then.walk_mut(f);
        self.r#else.walk_mut(f);
    }
}

impl EffectImpl for IfEffect {
    fn process(self: Box<Self>, context: LogicEffectContext, logic: &mut logic::Logic) {
        let effect = *self;
        let effect = if logic.model.check_condition(&effect.condition, &context) {
            effect.then
        } else {
            effect.r#else
        };
        // todo: use new queue
        // logic.effects.push_front(context, effect);
    }
}
