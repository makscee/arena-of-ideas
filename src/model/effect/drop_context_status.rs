use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DropContextStatusEffect {
    pub effect: Effect,
}

impl EffectContainer for DropContextStatusEffect {
    fn walk_effects_mut(&mut self, f: &mut dyn FnMut(&mut Effect)) {
        self.effect.walk_mut(f);
    }
}

impl EffectImpl for DropContextStatusEffect {
    fn process(self: Box<Self>, context: EffectContext, logic: &mut logic::Logic) {
        let effect = *self;
        logic.effects.push_front(
            EffectContext {
                status_id: None,
                ..context
            },
            effect.effect,
        );
    }
}
