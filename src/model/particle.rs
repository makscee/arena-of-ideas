use super::*;

#[derive(Clone, HasId)]
pub struct Particle {
    pub id: Id,
    pub parent: Option<Id>,
    pub partner: Option<Id>,
    pub position: Vec2<R32>,
    pub radius: R32,
    pub duration: Time,
    pub delay: Time,
    pub time_left: Time,
    pub render_config: ShaderConfig,
    pub follow: bool,
    pub color: Option<Rgba<f32>>,
    pub visible: bool,
}
