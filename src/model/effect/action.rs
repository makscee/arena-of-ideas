use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ActionEffect {
    pub time: Time,
}

impl EffectContainer for ActionEffect {
    fn walk_effects_mut(&mut self, f: &mut dyn FnMut(&mut Effect)) {}
}

impl EffectImpl for ActionEffect {
    fn process(self: Box<Self>, context: EffectContext, logic: &mut logic::Logic) {
        let effect = *self;
        let caster = logic.model.units.get_mut(&context.caster.unwrap()).unwrap();
        caster.last_action_time = logic.model.time;
        caster.action_state = ActionState::Start {
            time: effect.time,
            target: context.target.unwrap(),
        };
    }
}
