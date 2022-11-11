use super::*;

#[derive(Clone, HasId)]
pub struct Particle {
    pub id: Id,
    pub parent: Id,
    pub partner: Id,
    pub position: Vec2<R32>,
    pub radius: R32,
    pub duration: Time,
    pub delay: Time,
    pub time_left: Time,
    pub render_config: ShaderConfig,
    pub follow: bool,
    pub color: Rgba<f32>,
    pub visible: bool,
}
