use super::*;

pub struct HoverComponent {}

impl VarsProvider for HoverComponent {
    fn extend_vars(&self, vars: &mut Vars) {
        vars.insert(VarName::Hovered, Var::Float(1.0));
    }
}
