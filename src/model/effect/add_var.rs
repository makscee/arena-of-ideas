use super::*;

/// Adds a new variable to the context of the status with the name `status_name`
/// if it exists on the target
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct AddVarEffect {
    pub name: VarName,
    pub value: Expr,
    pub status_name: StatusName,
    pub caster: Option<Who>,
    #[serde(default)]
    pub effect: Effect,
}

impl EffectContainer for AddVarEffect {
    fn walk_effects_mut(&mut self, f: &mut dyn FnMut(&mut Effect)) {}
}

impl EffectImpl for AddVarEffect {
    fn process(self: Box<Self>, mut context: EffectContext, logic: &mut Logic) {
        let effect = *self;
        let value = effect.value.calculate(&context, logic);
        let target = context
            .target
            .and_then(|id| logic.model.units.get_mut(&id))
            .expect("Target not found");
        for status in target.all_statuses.iter_mut().filter(|status| {
            status.status.name == effect.status_name
                && effect
                    .caster
                    .map(|caster| context.get(caster) == status.caster)
                    .unwrap_or(true)
        }) {
            status.vars.insert(effect.name.clone(), value);
        }

        logic.effects.push_front(QueuedEffect {
            effect: effect.effect,
            context: {
                context.vars.insert(effect.name, value);
                context
            },
        });
    }
}
