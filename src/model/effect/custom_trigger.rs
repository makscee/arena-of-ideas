use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct CustomTriggerEffect {
    name: String,
}

impl EffectContainer for CustomTriggerEffect {
    fn walk_effects_mut(&mut self, f: &mut dyn FnMut(&mut Effect)) {}
}

impl EffectImpl for CustomTriggerEffect {
    fn process(self: Box<Self>, context: EffectContext, logic: &mut logic::Logic) {
        let effect = *self;
        for unit in &logic.model.units {
            for (effect, trigger, mut vars, status_id, status_color) in unit
                .all_statuses
                .iter()
                .filter(|status| {
                    if let Some(status_id) = context.status_id {
                        status.id == status_id
                    } else {
                        true
                    }
                })
                .flat_map(|status| {
                    status.trigger(|trigger| match trigger {
                        StatusTriggerType::Custom { name } => *name == effect.name,
                        _ => false,
                    })
                })
            {
                logic.effects.push_back(QueuedEffect {
                    effect,
                    context: EffectContext {
                        caster: Some(unit.id),
                        from: context.caster,
                        target: context.target,
                        vars: {
                            vars.extend(context.vars.clone());
                            vars
                        },
                        status_id: Some(status_id),
                        color: Some(status_color),
                    },
                })
            }
        }
    }
}
