use super::*;

#[derive(Clone, HasId)]
pub struct Particle {
    pub id: Id,
    pub parent: Option<Id>,
    pub position: Position,
    pub radius: Coord,
    pub duration: Time,
    pub time_left: Time,
    pub render_config: RenderConfig,
}
