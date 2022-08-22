use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", deny_unknown_fields, rename = "If")]
pub struct IncrVisualTimerEffect {
    pub value: f32,
}

impl EffectContainer for IncrVisualTimerEffect {
    fn walk_effects_mut(&mut self, f: &mut dyn FnMut(&mut Effect)) {}
}

impl EffectImpl for IncrVisualTimerEffect {
    fn process(self: Box<Self>, context: EffectContext, logic: &mut logic::Logic) {
        logic.model.current_tick.visual_timer += Time::new(self.value);
    }
}
