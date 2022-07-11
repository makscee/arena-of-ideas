use super::*;

#[derive(Clone, HasId)]
pub struct Particle {
    pub id: Id,
    pub parent: Option<Id>,
    pub partner: Option<Id>,
    pub position: Vec2<Coord>,
    pub radius: Coord,
    pub duration: Time,
    pub delay: Time,
    pub time_left: Time,
    pub render_config: RenderConfig,
    pub follow: bool,
}
