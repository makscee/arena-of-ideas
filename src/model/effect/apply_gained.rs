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
        caster
            .attached_statuses
            .retain(|status| match &status.status {
                Status::Gain(effect) => {
                    logic.effects.push_front(QueuedEffect {
                        effect: effect.clone(),
                        context,
                    });
                    false
                }
                _ => true,
            });
    }
}
