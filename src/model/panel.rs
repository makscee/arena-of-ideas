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
    pub fn create(text: String, duration: RealImpl<f32>, color: Option<Rgba<f32>>) -> Effect {
        Effect::Panel(Box::new(PanelEffect {
            duration,
            text,
            queue_delay: duration,
            color,
        }))
    }
}
