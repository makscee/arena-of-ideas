use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AddVarEffect {
    pub name: VarName,
    pub value: Expr,
    pub effect: Effect,
}

impl EffectContainer for AddVarEffect {
    fn walk_effects_mut(&mut self, f: &mut dyn FnMut(&mut Effect)) {}
}

impl EffectImpl for AddVarEffect {
    fn process(self: Box<Self>, context: EffectContext, logic: &mut Logic) {
        let effect = self.effect.clone();
        let value = self.value.calculate(&context, logic);
        logic.effects.push_front(QueuedEffect { effect, context: {
            let mut context = context.clone();
            context.vars.insert(self.name.clone(), value);
            context
        }, });
    }
}
