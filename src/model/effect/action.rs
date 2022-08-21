use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ActionEffect {}

impl EffectContainer for ActionEffect {
    fn walk_effects_mut(&mut self, f: &mut dyn FnMut(&mut Effect)) {}
}

impl EffectImpl for ActionEffect {
    fn process(self: Box<Self>, context: EffectContext, logic: &mut logic::Logic) {
        logic.process_turn();
    }
}
