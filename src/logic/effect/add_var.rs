use super::*;

/// Adds a new variable to the context of the status with the name `status_name`
/// if it exists on the target
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct AddVarEffect {
    pub name: VarName,
    pub value: Expr,
    pub creator: Option<Who>,
    #[serde(default)]
    pub effect: LogicEffect,
}

impl EffectContainer for AddVarEffect {
    fn walk_effects_mut(&mut self, f: &mut dyn FnMut(&mut LogicEffect)) {}
}

impl EffectImpl for AddVarEffect {
    fn process(self: Box<Self>, mut context: LogicEffectContext, logic: &mut Logic) {
        let effect = *self;
        let value = effect.value.calculate(&context, &logic.model);
        let target = logic
            .model
            .units
            .get_mut(&context.target)
            .expect("Target not found");
        // todo: use new queue
        // logic.effects.push_front(
        //     {
        //         context.vars.insert(effect.name, value);
        //         context
        //     },
        //     effect.effect,
        // );
    }
}
