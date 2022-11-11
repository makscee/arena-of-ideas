use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ApplyGainedEffect {}

impl EffectContainer for ApplyGainedEffect {
    fn walk_effects_mut(&mut self, f: &mut dyn FnMut(&mut Effect)) {}
}

impl EffectImpl for ApplyGainedEffect {
    fn process(self: Box<Self>, context: EffectContext, logic: &mut logic::Logic) {
        let effect = *self;
        let owner = logic.model.get(Who::Owner, &context);
        // TODO: remove these statuses immediately after application
        for (effect, trigger, mut vars, status_id, status_color) in
            owner.all_statuses.iter().flat_map(|status| {
                status.trigger(|trigger| matches!(trigger, StatusTrigger::GainedEffect))
            })
        {
            logic.effects.push_front(
                EffectContext {
                    vars: {
                        vars.extend(context.vars.clone());
                        vars
                    },
                    status_id: Some(status_id),
                    color: status_color,
                    ..context.clone()
                },
                effect,
            )
        }
    }
}
