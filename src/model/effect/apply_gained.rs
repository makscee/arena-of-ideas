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
        let owner = logic.model.get_who(Who::Owner, &context);
        // TODO: remove these statuses immediately after application
        for trigger_effect in owner
            .trigger()
            .filter(|effect| matches!(effect.trigger, StatusTrigger::GainedEffect))
        {
            let mut vars = trigger_effect.vars.clone();
            vars.extend(context.vars.clone());
            logic.effects.push_front(
                EffectContext {
                    vars,
                    status_id: Some(trigger_effect.status_id),
                    color: trigger_effect.status_color,
                    ..context.clone()
                },
                trigger_effect.effect,
            )
        }
    }
}
