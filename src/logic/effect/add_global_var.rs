use super::*;

/// Adds a new global variable that will be inserted in context of any effect if that VarName is absent
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct AddGlobalVarEffect {
    pub name: VarName,
    pub value: Expr,
}

impl EffectImpl for AddGlobalVarEffect {
    fn process(self: Box<Self>, mut context: LogicEffectContext, logic: &mut Logic) {
        let effect = *self;
        let value = effect.value.calculate(&context, &logic.model);
        logic.model.vars.insert(effect.name, value);
    }
}

impl EffectContainer for AddGlobalVarEffect {
    fn walk_effects_mut(&mut self, _f: &mut dyn FnMut(&mut LogicEffect)) {}
}
