use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ListEffect {
    pub effects: Vec<Effect>,
}

impl EffectContainer for ListEffect {
    fn walk_effects_mut(&mut self, f: &mut dyn FnMut(&mut Effect)) {
        for effect in &mut self.effects {
            effect.walk_mut(f);
        }
    }
}

impl EffectImpl for ListEffect {
    fn process(self: Box<Self>, context: EffectContext, logic: &mut logic::Logic) {
        let effect = *self;
        for effect in effect.effects.into_iter().rev() {
            logic.effects.push_front(QueuedEffect {
                effect,
                context: context.clone(),
            });
        }
    }
}
