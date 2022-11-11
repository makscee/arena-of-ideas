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
        let target_id = effect.who.and_then(|who| Some(context.get_id(who)));
        for unit in &logic.model.units {
            if target_id
                .and_then(|id| Some(id != unit.id))
                .or(Some(false))
                .unwrap()
            {
                continue;
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
                        StatusTrigger::Custom { name } => *name == effect.name,
                        _ => false,
                    })
                })
            {
                logic.effects.push_back(
                    EffectContext {
                        creator: context.owner,
                        owner: unit.id,
                        vars: {
                            vars.extend(context.vars.clone());
                            vars
                        },
                        status_id: None,
                        color: status_color,
                        ..context.clone()
                    },
                    effect,
                )
            }
        }
    }
}
