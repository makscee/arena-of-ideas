use super::*;

#[derive(Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct VisualEffect {
    #[serde(rename = "render")]
    pub render_config: RenderConfig,
    #[serde(skip)]
    pub render_mode: RenderMode,
}

impl std::fmt::Debug for VisualEffect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VisualEffect")
            .field("render_config", &self.render_config)
            .finish()
    }
}

impl EffectContainer for VisualEffect {
    fn walk_effects_mut(&mut self, f: &mut dyn FnMut(&mut Effect)) {}
}

impl EffectImpl for VisualEffect {
    fn process(self: Box<Self>, context: EffectContext, logic: &mut logic::Logic) {
        let effect = *self;
        let caster = logic.model.units.get_mut(&context.caster.unwrap()).unwrap();
        caster
            .attached_statuses
            .retain(|status| match &status.status {
                Status::GainedEffect(status) => {
                    logic.effects.push_front(QueuedEffect {
                        effect: status.effect.clone(),
                        context: context.clone(),
                    });
                    false
                }
                _ => true,
            });
    }
}
