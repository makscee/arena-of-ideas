use super::*;

#[derive(HasId, Clone)]
pub struct Projectile {
    pub id: Id,
    pub caster: Id,
    pub target: Id,
    pub target_position: Vec2<R32>,
    pub position: Vec2<R32>,
    pub speed: R32,
    pub effect: Effect,
    pub render_config: RenderConfig,
    pub vars: HashMap<VarName, R32>,
}
