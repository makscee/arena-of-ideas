use super::*;

#[derive(Clone, HasId)]
pub struct Panel {
    pub id: Id,
    pub text: String,
    pub duration: Time,
    pub time_passed: Time,
    pub color: Rgba<f32>,
    pub visible: bool,
}

impl Panel {
    pub fn create(
        effects: &mut EffectOrchestrator,
        text: String,
        duration: RealImpl<f32>,
        color: Option<Rgba<f32>>,
    ) {
        effects.push_front(
            EffectContext::empty(),
            Effect::Panel(Box::new(PanelEffect {
                duration,
                text,
                queue_delay: duration,
                color,
            })),
        );
    }
}
