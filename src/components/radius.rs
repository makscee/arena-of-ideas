use super::*;

#[derive(Clone)]
pub struct RadiusComponent(pub f32);

impl VarsProvider for RadiusComponent {
    fn extend_vars(&self, vars: &mut Vars) {
        vars.insert(VarName::Radius, Var::Float(self.0));
    }
}

impl Default for RadiusComponent {
    fn default() -> Self {
        Self(1.0)
    }
}
