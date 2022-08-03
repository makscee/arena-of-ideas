use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct VisualEffect {
    pub duration: Time,
    #[serde(default = "default_delay")]
    pub delay: Time,
    #[serde(default = "default_parent")]
    pub parent: Who,
    #[serde(default = "default_partner")]
    pub partner: Who,
    #[serde(default = "default_follow")]
    pub follow: bool,
    pub radius: R32,
    #[serde(default = "default_color")]
    pub color: Option<Color<f32>>,
    #[serde(rename = "render")]
    pub render_config: ShaderConfig,
}

fn default_delay() -> Time {
    Time::new(0.0)
}

fn default_follow() -> bool {
    false
}

fn default_parent() -> Who {
    Who::Target
}

fn default_partner() -> Who {
    Who::Caster
}

fn default_color() -> Option<Color<f32>> {
    None
}

impl EffectContainer for VisualEffect {
    fn walk_effects_mut(&mut self, f: &mut dyn FnMut(&mut Effect)) {}
}

impl EffectImpl for VisualEffect {
    fn process(self: Box<Self>, context: EffectContext, logic: &mut logic::Logic) {
        let effect = *self;

        let parent = context.get(effect.parent);
        let position = parent
            .and_then(|parent| {
                logic
                    .model
                    .units
                    .get(&parent)
                    .or(logic.model.dead_units.get(&parent))
            })
            .map(|unit| unit.position);
        let mut effect_color = None;
        if let Some(color) = effect.color {
            effect_color = Some(color);
        } else if let Some(color) = context.color {
            effect_color = Some(color);
        }
        if let Some(position) = position {
            let partner = context.get(effect.partner);

            logic.model.particles.insert(Particle {
                id: logic.model.next_id,
                radius: effect.radius,
                duration: effect.duration,
                delay: effect.delay,
                time_left: effect.duration,
                render_config: effect.render_config,
                parent,
                partner,
                position: position.to_world(),
                follow: effect.follow,
                color: effect_color,
            });
            logic.model.next_id += 1;
        }
    }
}
