use super::*;

/// Adds a new variable to the context of the status with the name `status_name`
/// if it exists on the target
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct AddVarEffect {
    pub name: VarName,
    pub value: Expr,
    #[serde(default)]
    pub status_name: Option<StatusName>,
    pub creator: Option<Who>,
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
        let target = logic
            .model
            .units
            .get_mut(&context.target)
            .or(logic.model.dead_units.get_mut(&context.target))
            .expect("Target not found");
        if let Some(status_name) = effect.status_name {
            for status in target.all_statuses.iter_mut().filter(|status| {
                status.status.name == status_name
                    && effect
                        .creator
                        .map(|creator| context.get_id(creator) == status.creator)
                        .unwrap_or(true)
            }) {
                status.vars.insert(effect.name.clone(), value);
            }
        } else if let Some(status_id) = context.status_id {
            for status in target.all_statuses.iter_mut().filter(|status| {
                status.id == status_id
                    && effect
                        .creator
                        .map(|creator| context.get_id(creator) == status.creator)
                        .unwrap_or(true)
            }) {
                status.vars.insert(effect.name.clone(), value);
            }
        }

        logic.effects.push_front(
            {
                context.vars.insert(effect.name, value);
                context
            },
            effect.effect,
        );
    }
}
