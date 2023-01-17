use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KillEffect {
    pub who: Who,
}

impl EffectContainer for KillEffect {
    fn walk_effects_mut(&mut self, _f: &mut dyn FnMut(&mut LogicEffect)) {}
}

impl EffectImpl for KillEffect {
    fn process(self: Box<Self>, context: LogicEffectContext, logic: &mut logic::Logic) {
        let id = context.get_id(self.who);
        if logic.model.units.get(&id).is_some() {
            // logic.kill(id, context);
        }
    }
}
