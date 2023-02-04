use super::*;

pub struct Position(pub vec2<f32>);

impl VarsProvider for Position {
    fn extend_vars(&self, vars: &mut Vars) {
        vars.insert(VarName::Position, Var::Vec2(self.0));
    }
}

impl Default for Position {
    fn default() -> Self {
        Self(vec2::ZERO)
    }
}
