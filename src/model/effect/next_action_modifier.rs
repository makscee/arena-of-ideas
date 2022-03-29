use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct NextActionModifierEffect {
    pub modifier: Modifier,
}

impl EffectContainer for NextActionModifierEffect {
    fn walk_effects_mut(&mut self, f: &mut dyn FnMut(&mut Effect)) {}
}

impl EffectImpl for NextActionModifierEffect {
    fn process(self: Box<Self>, context: EffectContext, logic: &mut logic::Logic) {
        let effect = *self;
        let target = logic.model.units.get_mut(&context.target.unwrap()).unwrap();
        target.next_action_modifiers.push(effect.modifier);
    }
}
