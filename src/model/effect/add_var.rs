use super::*;

/// Adds a new variable to the context of the status with the name `status_name`
/// if it exists on the target
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AddVarEffect {
    pub name: VarName,
    pub value: Expr,
    pub status_name: StatusName,
}

impl EffectContainer for AddVarEffect {
    fn walk_effects_mut(&mut self, f: &mut dyn FnMut(&mut Effect)) {}
}

impl EffectImpl for AddVarEffect {
    fn process(self: Box<Self>, context: EffectContext, logic: &mut Logic) {
        let effect = *self;
        let value = effect.value.calculate(&context, logic);
        let target = context
            .target
            .and_then(|id| logic.model.units.get_mut(&id))
            .expect("Target not found");
        for status in target.all_statuses.iter_mut().filter(|status| status.status.name == effect.status_name) {
            status.vars.insert(effect.name.clone(), value);
        }
    }
}
