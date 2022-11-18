use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct PanelEffect {
    pub duration: Time,
    pub text: String,
    #[serde(default = "default_delay")]
    pub queue_delay: Time, // all queue effects processing will be stopped for this time
    #[serde(default = "default_color")]
    pub color: Option<Rgba<f32>>,
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
    Who::Owner
}

fn default_color() -> Option<Rgba<f32>> {
    None
}

impl EffectContainer for PanelEffect {
    fn walk_effects_mut(&mut self, f: &mut dyn FnMut(&mut Effect)) {}
}

impl EffectImpl for PanelEffect {
    fn process(self: Box<Self>, context: EffectContext, logic: &mut logic::Logic) {
        let effect = *self;

        logic.model.render_model.panels.insert(Panel {
            id: logic.model.next_id,
            text: effect.text,
            duration: effect.duration,
            time_passed: r32(0.0),
            color: match effect.color {
                Some(color) => color,
                None => Rgba::WHITE,
            },
            visible: true,
        });
        logic.model.next_id += 1;
        logic
            .effects
            .add_delay(&context, effect.queue_delay.as_f32());
    }
}
