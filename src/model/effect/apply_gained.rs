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
        let caster = logic.model.units.get_mut(&context.caster.unwrap()).unwrap();
        // TODO: remove these statuses immediately after application
        for (effect, mut vars, status_id, status_color) in
            caster.all_statuses.iter().flat_map(|status| {
                status.trigger(|trigger| matches!(trigger, StatusTrigger::GainedEffect))
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
