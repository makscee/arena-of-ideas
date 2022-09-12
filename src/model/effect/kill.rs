use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KillEffect {
    pub who: Who,
}

impl EffectContainer for KillEffect {
    fn walk_effects_mut(&mut self, _f: &mut dyn FnMut(&mut Effect)) {}
}

impl EffectImpl for KillEffect {
    fn process(self: Box<Self>, context: EffectContext, logic: &mut logic::Logic) {
        if let Some(target) = context
            .get(self.who)
            .and_then(|id| logic.model.units.get_mut(&id))
        {
            let id = target.id;
            logic.kill(id);
        }
    }
}
