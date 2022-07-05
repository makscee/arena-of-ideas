use super::*;

#[derive(HasId, Clone)]
pub struct Projectile {
    pub id: Id,
    pub caster: Id,
    pub target: Id,
    pub target_position: Position,
    pub position: Position,
    pub speed: Coord,
    pub effect: Effect,
    pub render_config: RenderConfig,
    pub vars: HashMap<VarName, R32>,
}
