use super::*;

#[derive(Clone, Debug)]
pub struct PositionComponent(pub vec2<f32>);

impl VarsProvider for PositionComponent {
    fn extend_vars(&self, vars: &mut Vars) {
        vars.insert(VarName::Position, Var::Vec2(self.0));
    }
}

impl Default for PositionComponent {
    fn default() -> Self {
        Self(vec2::ZERO)
    }
}
