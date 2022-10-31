use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct IncrVisualTimerEffect {
    pub value: Option<f32>,
}

impl EffectContainer for IncrVisualTimerEffect {
    fn walk_effects_mut(&mut self, f: &mut dyn FnMut(&mut Effect)) {}
}

impl EffectImpl for IncrVisualTimerEffect {
    fn process(self: Box<Self>, context: EffectContext, logic: &mut logic::Logic) {
        let value = self.value.unwrap_or(UNIT_TURN_TIME);
        logic.model.visual_timer += Time::new(value);
    }
}
