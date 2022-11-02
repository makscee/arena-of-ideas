use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct CustomTriggerEffect {
    name: String,
    who: Option<Who>,
}

impl EffectContainer for CustomTriggerEffect {
    fn walk_effects_mut(&mut self, f: &mut dyn FnMut(&mut Effect)) {}
}

impl EffectImpl for CustomTriggerEffect {
    fn process(self: Box<Self>, context: EffectContext, logic: &mut logic::Logic) {
        let effect = *self;
        let name = effect.name.clone();
        let mut target_unit: Option<Id> = None;
        if let Some(target) = effect.who {
            target_unit = context.get(target);
        }
        for unit in &logic.model.units {
            if let Some(target_unit) = target_unit {
                if unit.id != target_unit {
                    continue;
                }
            }
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
                logic.effects.push_front(QueuedEffect {
                    effect,
                    context: EffectContext {
                        caster: Some(unit.id),
                        from: context.caster,
                        target: context.target,
                        vars: {
                            vars.extend(context.vars.clone());
                            vars
                        },
                        status_id: None,
                        color: Some(status_color),
                    },
                })
            }
        }
    }
}
