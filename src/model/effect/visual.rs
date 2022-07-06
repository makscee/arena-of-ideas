use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct VisualEffect {
    pub duration: Time,
    #[serde(default = "default_parent")]
    pub parent: Who,
    #[serde(default = "default_follow")]
    pub follow: bool,
    pub radius: R32,
    #[serde(rename = "render")]
    pub render_config: RenderConfig,
}

fn default_follow() -> bool {
    false
}

fn default_parent() -> Who {
    Who::Target
}

impl EffectContainer for VisualEffect {
    fn walk_effects_mut(&mut self, f: &mut dyn FnMut(&mut Effect)) {}
}

impl EffectImpl for VisualEffect {
    fn process(self: Box<Self>, context: EffectContext, logic: &mut logic::Logic) {
        let effect = *self;

        let parent = context.get(effect.parent);
        let position = parent
            .and_then(|parent| logic.model.units.get(&parent))
            .map(|unit| unit.position)
            .expect("Parent not found");
        let parent = parent.filter(|_| effect.follow);

        logic.model.particles.insert(Particle {
            id: logic.model.next_id,
            radius: effect.radius,
            duration: effect.duration,
            time_left: effect.duration,
            render_config: effect.render_config,
            parent,
            position: position.to_world(),
        });
        logic.model.next_id += 1;
    }
}
