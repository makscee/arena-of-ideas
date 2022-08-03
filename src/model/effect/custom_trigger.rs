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
        let target = logic.model.units.get_mut(&context.target.unwrap()).unwrap();
        for (effect, mut vars, status_id, status_color) in target
            .all_statuses
            .iter()
            // .filter(|status| {
            //     if let Some(status_id) = context.status_id {
            //         status.id == status_id
            //     } else {
            //         true
            //     }
            // })
            .flat_map(|status| {
                status.trigger(|trigger| match trigger {
                    StatusTrigger::Custom { name } => *name == effect.name,
                    _ => false,
                })
            })
        {
            logic.effects.push_front(QueuedEffect {
                effect,
                context: EffectContext {
                    caster: context.caster,
                    from: context.from,
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
