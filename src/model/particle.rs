use super::*;

#[derive(Clone, HasId)]
pub struct Particle {
    pub id: Id,
    pub parent: Option<Id>,
    pub position: Vec2<Coord>,
    pub radius: Coord,
    pub time_left: Option<Time>,
    pub render_config: RenderConfig,
}
