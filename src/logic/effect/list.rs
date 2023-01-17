use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ListEffect {
    pub effects: Vec<LogicEffect>,
}

impl EffectContainer for ListEffect {
    fn walk_effects_mut(&mut self, f: &mut dyn FnMut(&mut LogicEffect)) {
        for effect in &mut self.effects {
            effect.walk_mut(f);
        }
    }
}

impl EffectImpl for ListEffect {
    fn process(self: Box<Self>, context: LogicEffectContext, logic: &mut logic::Logic) {
        let effect = *self;
        for effect in effect.effects.into_iter().rev() {
            // todo: use new queue
            // logic.effects.push_front(context.clone(), effect);
        }
    }
}
