use super::*;

#[derive(HasId, Clone)]
pub struct Projectile {
    pub id: Id,
    pub caster: Id,
    pub target: Id,
    pub target_position: Vec2<Coord>,
    pub position: Vec2<Coord>,
    pub speed: Coord,
    pub effect: Effect,
    pub render_config: RenderConfig,
}
